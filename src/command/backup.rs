use anyhow::Result;
use sekai_app::{BackupReport, HostWorldTree};

use crate::Context;

#[poise::command(slash_command, owners_only)]
pub async fn backup(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;

    let (world, store) = super::open_world()?;

    let Ok(_guard) = ctx.data().store_lock.try_lock() else {
        ctx.say("another task is already running").await?;
        return Ok(());
    };

    let report = run(world, &store).await?;
    let res = format!("new blobs: {}", report.new_blobs);

    ctx.say("backup completed").await?;
    ctx.say(res).await?;

    Ok(())
}

async fn run(mut world: HostWorldTree, store: &str) -> Result<BackupReport> {
    let mut instance = sekai_app::SekaiInstance::init(store).await?;

    let (report, _) = instance
        .world_mut(&mut world)
        .backup(
            sekai_app::BackupOptions::default(),
            sekai_app::Scope::World,
            |_| {},
        )
        .await?;

    Ok(report)
}
