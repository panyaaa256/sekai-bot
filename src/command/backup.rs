use anyhow::{Result, bail};
use sekai_app::{BackupOptions, BackupReport, HostWorldTree, Scope};

use crate::Context;
use crate::config::Config;

#[poise::command(slash_command, owners_only)]
pub async fn backup(ctx: Context<'_>, name: Option<String>) -> Result<()> {
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

    let report = run(world, &store, scope).await?;
    let res = format!("new blobs: {}", report.new_blobs);

    ctx.say("backup completed").await?;
    ctx.say(res).await?;

    Ok(())
}

async fn run(mut world: HostWorldTree, store: &str, scope: Scope) -> Result<BackupReport> {
    let mut instance = sekai_app::SekaiInstance::init(store).await?;

    let (report, _) = instance
        .world_mut(&mut world)
        .backup(BackupOptions::default(), scope, |_| {})
        .await?;

    Ok(report)
}
