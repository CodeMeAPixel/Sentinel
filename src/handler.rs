use log::{error, info, warn};
use poise::serenity_prelude::{CreateEmbed, CreateMessage, GuildId, UserId};
use sqlx::PgPool;

use crate::{core, Error};

pub async fn handle_mod_action(
    guild_id: GuildId,
    user_id: UserId,
    pool: &PgPool,
    ctx: &serenity::all::Context,
    action: core::UserLimitTypes,
    action_target: String,
) -> Result<(), Error> {
    // Skip whitelisted users/roles entirely: their actions are never recorded,
    // so they can neither contribute to nor trigger a limit.
    let member_roles = guild_id
        .member(&ctx, user_id)
        .await
        .map(|member| member.roles.clone())
        .unwrap_or_default();

    if core::is_whitelisted(pool, guild_id, user_id, &member_roles).await? {
        return Ok(());
    }

    // SAFETY: Tx should be dropped if error occurs, so make a scope to seperate tx queries
    {
        let mut tx = pool.begin().await?;

        // Look for guild
        let guild = sqlx::query!(
            "
            SELECT COUNT(*)
            FROM guilds
            WHERE guild_id = $1
        ",
            guild_id.to_string()
        )
        .fetch_one(&mut *tx)
        .await?;

        if guild.count.unwrap_or_default() == 0 {
            // Guild not found
            error!("Guild not found: {}", guild_id);
            return Ok(());
        }

        // Insert into user_actions
        sqlx::query!(
            "
            INSERT INTO user_actions (action_id, guild_id, user_id, limit_type, action_target)
            VALUES ($1, $2, $3, $4, $5)
        ",
            botox::crypto::gen_random(48),
            guild_id.to_string(),
            user_id.to_string(),
            action.to_string(),
            action_target
        )
        .execute(&mut *tx)
        .await?;

        // Check if they hit any limits yet
        let hit = core::CurrentUserLimitsHit::hit(guild_id, pool).await?;

        for hit_limit in hit {
            // We have a hit limit for this user
            info!("Hit limit: {:?}", hit_limit);

            // Immediately handle the limit
            let cur_uid = ctx.cache.current_user().id;
            let can_mod = {
                let guild = ctx.cache.guild(guild_id).ok_or("Guild not found")?;

                guild.greater_member_hierarchy(cur_uid, user_id)
            }
            .unwrap_or(cur_uid);

            if can_mod == cur_uid {
                info!("Moderating user");
                match hit_limit.limit.limit_action {
                    core::UserLimitActions::RemoveAllRoles => {
                        // Get all user roles
                        if let Ok(member) = guild_id.member(&ctx, user_id).await {
                            let roles = member.roles.clone();
                            for role in roles.iter() {
                                if let Err(e) = member.remove_role(
                                    &ctx.http, 
                                    *role, 
                                    Some("Removing roles due to preconfigured limits being hit")
                                ).await {
                                    error!("Failed to remove role: {}", e);
                                }
                            }
                        }
                    }
                    core::UserLimitActions::KickUser => {
                        if let Err(e) = guild_id.kick(&ctx.http, user_id, Some("Configured limit has been hit")).await {
                            error!("Failed to kick user: {}", e);
                        }
                    }
                    core::UserLimitActions::BanUser => {
                        if let Err(e) = guild_id.ban(&ctx.http, user_id, 0, Some("Configured limit has been hit")).await {
                            error!("Failed to kick user: {}", e);
                        }
                    }
                    core::UserLimitActions::TimeoutUser => {
                        let duration_secs = hit_limit
                            .limit
                            .limit_timeout_duration
                            .as_ref()
                            .map(crate::utils::pg_interval_seconds)
                            .unwrap_or(3600);

                        let until = sqlx::types::chrono::Utc::now().timestamp() + duration_secs;

                        match poise::serenity_prelude::Timestamp::from_unix_timestamp(until) {
                            Ok(timestamp) => {
                                if let Ok(mut member) = guild_id.member(&ctx, user_id).await {
                                    if let Err(e) = member
                                        .disable_communication_until(&ctx.http, timestamp)
                                        .await
                                    {
                                        error!("Failed to timeout user: {}", e);
                                    }
                                }
                            }
                            Err(e) => error!("Failed to build timeout timestamp: {}", e),
                        }
                    }
                }
            } else {
                warn!(
                    "Cannot moderate user, not enough permissions: {}, {}",
                    can_mod, cur_uid
                );

                sqlx::query!(
                    "
                INSERT INTO past_hit_limits
                (id, guild_id, user_id, limit_id, cause, notes)
                VALUES ($1, $2, $3, $4, $5, $6)",
                    botox::crypto::gen_random(16),
                    guild_id.to_string(),
                    user_id.to_string(),
                    hit_limit.limit.limit_id,
                    &hit_limit
                        .cause
                        .iter()
                        .map(|a| a.action_id.clone())
                        .collect::<Vec<_>>(),
                    &vec!["Not enough permissions to moderate user".to_string()]
                )
                .execute(&mut *tx)
                .await?;

                return Ok(());
            }

            for action in hit_limit.cause.iter() {
                sqlx::query!(
                    "
                UPDATE user_actions
                SET limits_hit = array_append(limits_hit, $1)
                WHERE action_id = $2",
                    hit_limit.limit.limit_id,
                    action.action_id
                )
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query!(
                "
            INSERT INTO past_hit_limits
            (id, guild_id, user_id, limit_id, cause)
            VALUES ($1, $2, $3, $4, $5)",
                botox::crypto::gen_random(16),
                guild_id.to_string(),
                user_id.to_string(),
                hit_limit.limit.limit_id,
                &hit_limit
                    .cause
                    .iter()
                    .map(|a| a.action_id.clone())
                    .collect::<Vec<_>>()
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
    }

    let settings = sqlx::query_as::<_, (Option<String>, bool, i32)>(
        "SELECT logs_channel_id, logs_enabled, logs_color FROM guild_settings WHERE guild_id = $1",
    )
    .bind(guild_id.to_string())
    .fetch_optional(pool)
    .await?;

    if let Some((Some(channel_id), true, color)) = settings {
        if let Ok(channel_id) = channel_id.parse::<serenity::all::ChannelId>() {
            let embed = CreateEmbed::default()
                .title("Moderation audit event")
                .description(format!("**{}** by <@{}> on `{}`", action.to_cond(), user_id, action_target))
                .color(color.clamp(0, 0xFFFFFF) as u32);

            if let Err(error) = channel_id
                .send_message(ctx, CreateMessage::new().embed(embed))
                .await
            {
                warn!("Could not send audit log message: {}", error);
            }
        }
    }

    Ok(())
}
