use rand::seq::SliceRandom;
use serenity::{all::OnlineStatus, gateway::ActivityData};

enum Status {
    Watch,
    Play,
    Listen,
}

pub async fn update_status(
    ctx: &serenity::all::Context,
) -> Result<(), crate::Error> {
    let statuses = [
        (Status::Watch, "/help"),
        (Status::Play, "stopping raids"),
        (Status::Listen, "listening to audits"),
    ];

    let (status, text) = statuses.choose(&mut rand::thread_rng()).unwrap();

    let activity = match status {
        Status::Watch => Some(ActivityData::watching(text.to_string())),
        Status::Play => Some(ActivityData::playing(text.to_string())),
        Status::Listen => Some(ActivityData::listening(text.to_string())),
    };

    // With autosharding, `ctx` only belongs to one shard; update every shard's
    // presence directly through the shard manager so the rotation is consistent
    // across the whole bot, not just the shard that happened to run this task.
    if let Some(shard_manager) = crate::SHARD_MANAGER.get() {
        for info in shard_manager.runners.lock().await.values() {
            info.runner_tx.set_presence(activity.clone(), OnlineStatus::Online);
        }
    } else {
        ctx.set_presence(activity, OnlineStatus::Online);
    }

    Ok(())
}
