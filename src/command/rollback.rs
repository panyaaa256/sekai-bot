use anyhow::Result;
use sekai_app::{HostWorldTree, RollbackReport, SnapshotId};

use crate::Context;

#[poise::command(slash_command, owners_only)]
pub async fn rollback(ctx: Context<'_>, id: u64) -> Result<()> {
    ctx.defer().await?;

    let (world, store) = super::open_world()?;

    let Ok(_guard) = ctx.data().store_lock.try_lock() else {
        ctx.say("another task is already running").await?;
        return Ok(());
    };

    let report = run(world, &store, SnapshotId(id)).await?;
    let res = format!("restored chunks: {}", report.chunks_restored);

    ctx.say("rollback completed").await?;
    ctx.say(res).await?;

    Ok(())
}

async fn run(mut world: HostWorldTree, store: &str, id: SnapshotId) -> Result<RollbackReport> {
    let mut instance = sekai_app::SekaiInstance::open(store).await?;

    let (report, _) = instance
        .world_mut(&mut world)
        .rollback(
            id,
            sekai_app::RollbackOptions::default(),
            sekai_app::Scope::World,
            |_| {},
        )
        .await?;

    Ok(report)
}
