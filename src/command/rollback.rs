use anyhow::{Result, bail};
use sekai_app::{HostWorldTree, RollbackReport, Scope, SnapshotId};

use crate::{Context, config::Config};

#[poise::command(slash_command, owners_only)]
pub async fn rollback(ctx: Context<'_>, id: u64, name: Option<String>) -> Result<()> {
    ctx.defer().await?;

    let (world, store) = super::open_world()?;
    let config = Config::load()?;

    let scope = match name {
        Some(key) => {
            let Some(profile) = config.profiles.get(&key) else {
                bail!("profile not found: {key}");
            };
            profile.scope()
        }
        None => Scope::World,
    };

    let Ok(_guard) = ctx.data().store_lock.try_lock() else {
        ctx.say("another task is already running").await?;
        return Ok(());
    };

    let report = run(world, &store, SnapshotId(id), scope).await?;
    let res = format!("restored chunks: {}", report.chunks_restored);

    ctx.say("rollback completed").await?;
    ctx.say(res).await?;

    Ok(())
}

async fn run(
    mut world: HostWorldTree,
    store: &str,
    id: SnapshotId,
    scope: Scope,
) -> Result<RollbackReport> {
    let mut instance = sekai_app::SekaiInstance::open(store).await?;

    let (report, _) = instance
        .world_mut(&mut world)
        .rollback(id, sekai_app::RollbackOptions::default(), scope, |_| {})
        .await?;

    Ok(report)
}
