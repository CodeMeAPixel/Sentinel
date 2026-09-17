use crate::Context;

pub async fn limits_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    // Fetch all limits available
    let data = ctx.data();

    let guild_id = ctx.guild_id();

    if guild_id.is_none() {
        return Vec::new();
    }

    let guild_id = guild_id.unwrap();

    let limits = crate::core::Limit::from_guild(&data.pool, guild_id).await;

    if let Ok(limits) = limits {
        let mut choices = Vec::new();

        let partial = partial.to_ascii_lowercase();

        for limit in limits {
            if limit.limit_name.to_ascii_lowercase().contains(&partial) {
                choices.push(serenity::all::AutocompleteChoice::new(
                    limit.limit_name,
                    limit.limit_id,
                ));

                if choices.len() == 25 {
                    break;
                }
            }
        }

        return choices;
    }

    Vec::new()
}
