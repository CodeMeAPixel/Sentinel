use poise::{
    serenity_prelude::{CreateEmbed, Member, Role, User},
    CreateReply,
};
use serenity::{all::{ChannelId, UserId}, builder::CreateAttachment, prelude::Mentionable};

use crate::{Context, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// Permission management
#[poise::command(
    prefix_command,
    slash_command,
    subcommands("add_admin", "remove_admin")
)]
pub async fn perms(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add admin to the server
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn add_admin(ctx: Context<'_>, user: Member) -> Result<(), Error> {
    // Check that user is guild owner
    if ctx.author().id != ctx.guild().ok_or("Could not get guild id")?.owner_id {
        return Err("In order to protect against nukes, bypassing the bot and other secuity concerns, only guild owners can add new admins at this time.".into());
    }

    // Check if user is already an admin, if so return
    let count = sqlx::query!(
        "SELECT COUNT(*) FROM guild_admins WHERE guild_id = $1 AND user_id = $2",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        user.user.id.to_string()
    )
    .fetch_one(&ctx.data().pool)
    .await
    .map_err(|_| "Could not fetch guild admin status")?;

    if count.count.unwrap_or_default() > 0 {
        return Err("User is already an admin".into());
    }

    sqlx::query!(
        "INSERT INTO guild_admins (guild_id, user_id) VALUES ($1, $2)",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        user.user.id.to_string()
    )
    .execute(&ctx.data().pool)
    .await?;

    sqlx::query("INSERT INTO guild_settings (guild_id) VALUES ($1) ON CONFLICT (guild_id) DO NOTHING")
        .bind(ctx.guild_id().ok_or("Could not get guild id")?.to_string())
        .execute(&ctx.data().pool)
        .await?;

    ctx.say("Added admin successfully").await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn settings(
    ctx: Context<'_>,
    #[description = "Channel for audit notifications; omit to keep the current channel"]
    logs_channel: Option<ChannelId>,
    #[description = "Whether audit notifications are enabled"]
    logs_enabled: Option<bool>,
    #[description = "Embed color as hexadecimal (0xRRGGBB) or decimal"]
    logs_color: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Could not get guild id")?.to_string();
    let color = logs_color
        .map(|value| {
            u32::from_str_radix(value.trim_start_matches("0x"), 16)
                .or_else(|_| value.parse::<u32>())
                .map_err(|_| "logs_color must be hexadecimal (0xRRGGBB) or decimal")
        })
        .transpose()?;

    sqlx::query("INSERT INTO guild_settings (guild_id, logs_channel_id, logs_enabled, logs_color) VALUES ($1, $2, COALESCE($3, TRUE), COALESCE($4, 65280)) ON CONFLICT (guild_id) DO UPDATE SET logs_channel_id = COALESCE($2, guild_settings.logs_channel_id), logs_enabled = COALESCE($3, guild_settings.logs_enabled), logs_color = COALESCE($4, guild_settings.logs_color)")
        .bind(&guild_id)
        .bind(logs_channel.map(|channel| channel.to_string()))
        .bind(logs_enabled)
        .bind(color.map(|value| value as i32))
        .execute(&ctx.data().pool)
        .await?;

    ctx.say("Audit log settings updated.").await?;
    Ok(())
}

/// Remove admin from the server
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn remove_admin(ctx: Context<'_>, user: Member) -> Result<(), Error> {
    // Check that user is guild owner
    if ctx.author().id != ctx.guild().ok_or("Could not get guild id")?.owner_id {
        return Err("Only guild owners can remove admins".into());
    }

    // Check if user is not already an admin, if so return
    let count = sqlx::query!(
        "SELECT COUNT(*) FROM guild_admins WHERE guild_id = $1 AND user_id = $2",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        user.user.id.to_string()
    )
    .fetch_one(&ctx.data().pool)
    .await
    .map_err(|_| "Could not fetch guild admin status")?;

    if count.count.unwrap_or_default() == 0 {
        return Err("User is not already an admin?".into());
    }

    sqlx::query!(
        "DELETE FROM guild_admins WHERE guild_id = $1 AND user_id = $2",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        user.user.id.to_string()
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Removed admin successfully").await?;

    Ok(())
}

/// Limits base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands(
        "limits_add",
        "limits_edit",
        "limits_view",
        "limits_remove",
        "limits_guide",
        "hit_limits"
    )
)]
pub async fn limits(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Explain how moderation limits work
#[poise::command(prefix_command, slash_command, guild_only, rename = "guide")]
pub async fn limits_guide(ctx: Context<'_>) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title("How Sentinel limits work")
        .description(
            "Limits watch moderation events and apply a response when a moderator reaches a threshold."
        )
        .field(
            "Add a limit",
            "Use `/limits add` and choose a name, event type, number of infractions, time window, and response.",
            false,
        )
        .field(
            "Event types",
            "Role Create, Role Update, Role Remove, Channel Create, Channel Update, Channel Remove, Webhook Create, Webhook Remove, Emoji Create, Emoji Remove, Kick, Ban, and Unban.",
            false,
        )
        .field(
            "Threshold example",
            "`3` bans in `10` minutes means the response runs when the same moderator reaches three bans during that window.",
            false,
        )
        .field(
            "Responses",
            "Remove All Roles, Kick User, Ban User, or Timeout User.",
            false,
        )
        .field(
            "Manage limits",
            "`/limits view` lists configured limits. `/limits edit` updates one in place. Use `/limits remove` and select a limit from the suggestions.",
            false,
        )
        .field(
            "Whitelist",
            "`/whitelist add` exempts a trusted user or role from limit tracking entirely; `/whitelist view` and `/whitelist remove` manage it.",
            false,
        )
        .field(
            "History",
            "`/limits hit` shows triggered limits, while `/actions view` shows recorded moderation actions.",
            false,
        )
        .color(0x00ff00);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Add a limit to the server
#[poise::command(prefix_command, slash_command, guild_only, rename = "add")]
pub async fn limits_add(
    ctx: Context<'_>,
    #[description = "The name of the limit"]
    limit_name: String,
    #[description = "The type of limit to impose on moderators"]
    limit_type: crate::core::UserLimitTypesChoices,
    #[description = "The amount of times the limit can be hit"]
    limit_per: i32,
    #[description = "The time interval infractions are counted in"]
    limit_time: i64,
    #[description = "The time unit for the time interval [seconds/minutes/hours/days]"]
    limit_time_unit: crate::utils::Unit,
    #[description = "The action to take when the limit is hit"]
    limit_action: crate::core::UserLimitActionsChoices,
    #[description = "Timeout duration, only used when action is Timeout User (default 1 hour)"]
    timeout_duration: Option<i64>,
    #[description = "The time unit for the timeout duration [seconds/minutes/hours/days]"]
    timeout_duration_unit: Option<crate::utils::Unit>,
) -> Result<(), Error> {
    let limit_type = limit_type.resolve();
    let limit_action = limit_action.resolve();

    let timeout_duration_secs = timeout_duration
        .map(|value| value * timeout_duration_unit.unwrap_or(crate::utils::Unit::Seconds).to_seconds());

    // Add limit to db
    sqlx::query!(
        "
            INSERT INTO limits (
                guild_id,
                limit_name,
                limit_type,
                limit_action,
                limit_per,
                limit_time,
                limit_timeout_duration
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                make_interval(secs => $6),
                CASE WHEN $7::double precision IS NULL THEN NULL ELSE make_interval(secs => $7) END
            )
        ",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        limit_name,
        limit_type.to_string(),
        limit_action.to_string(),
        limit_per,
        (limit_time * limit_time_unit.to_seconds()) as f64,
        timeout_duration_secs.map(|value| value as f64)
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Added limit successfully").await?;

    Ok(())
}

/// Edit an existing limit
#[poise::command(prefix_command, slash_command, guild_only, rename = "edit")]
pub async fn limits_edit(
    ctx: Context<'_>,
    #[description = "The limit id to edit"]
    #[autocomplete = "crate::autocompletes::limits_autocomplete"]
    limit_id: String,
    #[description = "The new name of the limit"]
    limit_name: Option<String>,
    #[description = "The new type of limit to impose on moderators"]
    limit_type: Option<crate::core::UserLimitTypesChoices>,
    #[description = "The new amount of times the limit can be hit"]
    limit_per: Option<i32>,
    #[description = "The new time interval infractions are counted in"]
    limit_time: Option<i64>,
    #[description = "The time unit for the time interval [seconds/minutes/hours/days]"]
    limit_time_unit: Option<crate::utils::Unit>,
    #[description = "The new action to take when the limit is hit"]
    limit_action: Option<crate::core::UserLimitActionsChoices>,
    #[description = "New timeout duration, only used when action is Timeout User"]
    timeout_duration: Option<i64>,
    #[description = "The time unit for the timeout duration [seconds/minutes/hours/days]"]
    timeout_duration_unit: Option<crate::utils::Unit>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Could not get guild id")?.to_string();

    let count = sqlx::query!(
        "SELECT COUNT(*) FROM limits WHERE guild_id = $1 AND limit_id = $2",
        guild_id,
        limit_id
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    if count.count.unwrap_or_default() == 0 {
        return Err("Could not find limit".into());
    }

    let limit_time_secs = limit_time.map(|value| {
        (value * limit_time_unit.unwrap_or(crate::utils::Unit::Seconds).to_seconds()) as f64
    });
    let timeout_duration_secs = timeout_duration.map(|value| {
        (value * timeout_duration_unit.unwrap_or(crate::utils::Unit::Seconds).to_seconds()) as f64
    });

    sqlx::query!(
        "
            UPDATE limits SET
                limit_name = COALESCE($3, limit_name),
                limit_type = COALESCE($4, limit_type),
                limit_action = COALESCE($5, limit_action),
                limit_per = COALESCE($6, limit_per),
                limit_time = CASE WHEN $7::double precision IS NULL THEN limit_time ELSE make_interval(secs => $7) END,
                limit_timeout_duration = CASE WHEN $8::double precision IS NULL THEN limit_timeout_duration ELSE make_interval(secs => $8) END
            WHERE guild_id = $1
            AND limit_id = $2
        ",
        guild_id,
        limit_id,
        limit_name,
        limit_type.map(|value| value.resolve().to_string()),
        limit_action.map(|value| value.resolve().to_string()),
        limit_per,
        limit_time_secs,
        timeout_duration_secs
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Updated limit successfully").await?;

    Ok(())
}

/// View the limits setup for this server
#[poise::command(prefix_command, slash_command, guild_only, rename = "view")]
pub async fn limits_view(ctx: Context<'_>) -> Result<(), Error> {
    let limits = crate::core::Limit::from_guild(
        &ctx.data().pool,
        ctx.guild_id().ok_or("Could not get guild id")?,
    )
    .await?;

    if limits.is_empty() {
        ctx.say("No limits setup for this server, use ``/limits add`` to add one!")
            .await?;
        return Ok(());
    }

    let mut cr: CreateReply<'_> = CreateReply::default();

    let mut added: i32 = 0;
    let mut i = 0;
    let mut embeds = vec![];

    for limit in limits {
        added += 1;

        if added >= 15 {
            added = 0;
            i += 1;
        }

        if embeds.len() <= i {
            embeds.push(CreateEmbed::default().title("Limits").color(0x00ff00));
        }

        embeds[i] = embeds[i].clone().field(
            limit.limit_name,
            format!(
                "If over {amount} ``{cond}`` triggered between {time} interval: ``{then}`` [{id}]",
                amount = limit.limit_per,
                cond = limit.limit_type.to_cond(),
                time = crate::utils::parse_pg_interval(limit.limit_time),
                then = limit.limit_action.to_cond(),
                id = limit.limit_id
            ),
            false,
        );
    }

    for embed in embeds {
        cr = cr.embed(embed);
    }

    ctx.send(cr).await?;

    Ok(())
}

/// Remove a limit from the server
#[poise::command(prefix_command, slash_command, guild_only, rename = "remove")]
pub async fn limits_remove(
    ctx: Context<'_>,
    #[description = "The limit id to remove"]
    #[autocomplete = "crate::autocompletes::limits_autocomplete"]
    limit_id: String,
) -> Result<(), Error> {
    // Look for limit using COUNT
    let count = sqlx::query!(
        "
            SELECT COUNT(*) FROM limits
            WHERE guild_id = $1
            AND limit_id = $2
        ",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        limit_id
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    if count.count.unwrap_or_default() == 0 {
        return Err("Could not find limit".into());
    }

    // Remove limit
    sqlx::query!(
        "
            DELETE FROM limits
            WHERE guild_id = $1
            AND limit_id = $2
        ",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        limit_id
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Removed limit successfully").await?;

    Ok(())
}

/// Setup the bot if it is not already setup
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    // Check if guild is already setup
    let count = sqlx::query!(
        "SELECT COUNT(*) FROM guilds WHERE guild_id = $1",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string()
    )
    .fetch_one(&ctx.data().pool)
    .await
    .map_err(|_| "Could not fetch guild status")?;

    if count.count.unwrap_or_default() > 0 {
        return Err("Guild is already setup".into());
    }

    // Add guild to db
    sqlx::query!(
        "INSERT INTO guilds (guild_id) VALUES ($1)",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string()
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Setup successfully. Now you can add limits for Sentinel to monitor for")
        .await?;

    Ok(())
}

/// Action management
#[poise::command(prefix_command, slash_command, guild_only, subcommands("actions_view"))]
pub async fn actions(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View actions taken by users that have been recorded by Sentinel
#[poise::command(prefix_command, slash_command, guild_only, rename = "view")]
pub async fn actions_view(
    ctx: Context<'_>,
    #[description = "User ID (optional)"] user_id: Option<UserId>,
) -> Result<(), Error> {
    let actions = {
        if let Some(user_id) = user_id {
            crate::core::Action::user(
                &ctx.data().pool,
                ctx.guild_id().ok_or("Could not get guild id")?,
                user_id,
            )
            .await?
        } else {
            crate::core::Action::guild(
                &ctx.data().pool,
                ctx.guild_id().ok_or("Could not get guild id")?,
            )
            .await?
        }
    };

    if actions.is_empty() {
        ctx.say("No actions recorded").await?;
        return Ok(());
    }

    if actions.len() > 30 {
        let actions = serde_json::to_string(&actions).map_err(|_| "Could not serialize actions")?;

        // Create a attachment
        let attachment = CreateAttachment::bytes(actions.into_bytes(), "actions.json");

        ctx.send(CreateReply::default().attachment(attachment)).await?;

        return Ok(());
    }

    let mut cr = CreateReply::default();
    let mut embeds = vec![];
    let mut added: i32 = 0;
    let mut i = 0;

    for action in actions {
        added += 1;

        if added >= 8 {
            added = 0;
            i += 1;
        }

        if embeds.len() <= i {
            embeds.push(CreateEmbed::default().title("Actions").color(0x00ff00));
        }

        embeds[i] = embeds[i].clone().field(
            action.action_id.clone(),
            format!(
                "``{limit_type}`` on ``{action_target}`` by {user_id} at <t:{timestamp}:R> [{id}]\n**Hit Limits:** {limits_hit:#?}",
                limit_type = action.limit_type,
                action_target = action.action_target,
                user_id = action.user_id.mention().to_string() + " (" + &action.user_id.to_string() + ")",
                timestamp = action.created_at.timestamp(),
                id = action.action_id,
                limits_hit = action.limits_hit
            ),
            false,
        );
    }

    for embed in embeds {
        cr = cr.embed(embed);
    }

    ctx.send(cr).await?;

    Ok(())
}

/// View hit limits
#[poise::command(prefix_command, slash_command, guild_only, rename = "hit")]
pub async fn hit_limits(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let hit_limits = crate::core::PastHitLimits::guild(
        &ctx.data().pool,
        ctx.guild_id().ok_or("Could not get guild id")?,
    ).await?;        

    if hit_limits.is_empty() {
        ctx.say("No hit limits recorded").await?;
        return Ok(());
    }

    if hit_limits.len() > 30 {
        let hit_limits = serde_json::to_string(&hit_limits).map_err(|_| "Could not serialize hit_limits")?;

        // Create a attachment
        let attachment = CreateAttachment::bytes(hit_limits.into_bytes(), "hit_limits.json");

        ctx.send(CreateReply::default().attachment(attachment)).await?;

        return Ok(());
    }

    let mut cr = CreateReply::default();
    let mut embeds = vec![];
    let mut added: i32 = 0;
    let mut i = 0;

    for hit_limit in hit_limits {
        added += 1;

        if added >= 8 {
            added = 0;
            i += 1;
        }

        if embeds.len() <= i {
            embeds.push(CreateEmbed::default().title("Past Limits History").color(0x00ff00));
        }

        let mut notes = String::new();

        for note in hit_limit.notes {
            notes.push_str(&format!("- ``{}`` ", note));
        }

        let mut causes = String::new();

        for cause in hit_limit.cause {
            causes.push_str(
                &format!(
                    "``{limit_type}`` on ``{action_target}`` by {user_id} at <t:{timestamp}:R> [{id}]\n**Hit Limits:** {limits_hit:#?}",
                    limit_type = cause.limit_type,
                    action_target = cause.action_target,
                    user_id = cause.user_id.mention().to_string() + " (" + &cause.user_id.to_string() + ")",
                    timestamp = cause.created_at.timestamp(),
                    id = cause.action_id,
                    limits_hit = cause.limits_hit
                ),
            );
        }

        embeds[i] = embeds[i].clone().field(
            hit_limit.id.clone(),
            format!(
                "Limit ``{limit_id}`` reached by ``{user_id}`` at <t:{timestamp}:R> [{id}]\n**Notes:** {notes}\n**Causes:** {causes}",
                limit_id = hit_limit.limit_id,
                user_id = hit_limit.user_id.mention().to_string() + " (" + &hit_limit.user_id.to_string() + ")",
                timestamp = hit_limit.created_at.timestamp(),
                id = hit_limit.id,
                notes = notes,
                causes = causes
            ),
            false,
        );
    }

    for embed in embeds {
        cr = cr.embed(embed);
    }

    ctx.send(cr).await?;

    Ok(())
}
/// Whitelist management: exempt trusted users or roles from limit tracking
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("whitelist_add", "whitelist_remove", "whitelist_view")
)]
pub async fn whitelist(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add a user or role to the whitelist
#[poise::command(prefix_command, slash_command, guild_only, rename = "add")]
pub async fn whitelist_add(
    ctx: Context<'_>,
    #[description = "The user to exempt from limit tracking"] user: Option<User>,
    #[description = "The role to exempt from limit tracking"] role: Option<Role>,
) -> Result<(), Error> {
    let (entity_id, entity_type) = match (user, role) {
        (Some(user), None) => (user.id.to_string(), "user"),
        (None, Some(role)) => (role.id.to_string(), "role"),
        _ => return Err("Provide exactly one of `user` or `role`".into()),
    };

    let guild_id = ctx.guild_id().ok_or("Could not get guild id")?.to_string();

    sqlx::query!(
        "INSERT INTO guild_whitelist (guild_id, entity_id, entity_type) VALUES ($1, $2, $3) ON CONFLICT (guild_id, entity_id) DO NOTHING",
        guild_id,
        entity_id,
        entity_type
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Added to whitelist successfully").await?;

    Ok(())
}

/// Remove a user or role from the whitelist
#[poise::command(prefix_command, slash_command, guild_only, rename = "remove")]
pub async fn whitelist_remove(
    ctx: Context<'_>,
    #[description = "The user to remove from the whitelist"] user: Option<User>,
    #[description = "The role to remove from the whitelist"] role: Option<Role>,
) -> Result<(), Error> {
    let entity_id = match (user, role) {
        (Some(user), None) => user.id.to_string(),
        (None, Some(role)) => role.id.to_string(),
        _ => return Err("Provide exactly one of `user` or `role`".into()),
    };

    let guild_id = ctx.guild_id().ok_or("Could not get guild id")?.to_string();

    sqlx::query!(
        "DELETE FROM guild_whitelist WHERE guild_id = $1 AND entity_id = $2",
        guild_id,
        entity_id
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Removed from whitelist successfully").await?;

    Ok(())
}

/// View the whitelist for this server
#[poise::command(prefix_command, slash_command, guild_only, rename = "view")]
pub async fn whitelist_view(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Could not get guild id")?.to_string();

    let rec = sqlx::query!(
        "SELECT entity_id, entity_type FROM guild_whitelist WHERE guild_id = $1",
        guild_id
    )
    .fetch_all(&ctx.data().pool)
    .await?;

    if rec.is_empty() {
        ctx.say("No users or roles are whitelisted for this server").await?;
        return Ok(());
    }

    let mut description = String::new();

    for entry in rec {
        let mention = if entry.entity_type == "role" {
            format!("<@&{}>", entry.entity_id)
        } else {
            format!("<@{}>", entry.entity_id)
        };

        description.push_str(&format!("- {} (``{}``)\n", mention, entry.entity_type));
    }

    let embed = CreateEmbed::default()
        .title("Whitelisted users & roles")
        .description(description)
        .color(0x00ff00);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
