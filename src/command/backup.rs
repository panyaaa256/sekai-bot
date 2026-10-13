use anyhow::Result;
use poise::CreateReply;
use poise::serenity_prelude::CreateEmbed;
use sekai_app::{BackupOptions, BackupReport};
use tokio::sync::watch;

use crate::Context;
use crate::command::Progress;
use crate::config::Config;

#[poise::command(slash_command, owners_only)]
pub async fn backup(ctx: Context<'_>, name: Option<String>) -> Result<()> {
    ctx.defer().await?;

    let config = Config::load()?;
    let scope = config.resolve_scope(name)?;
    let (mut world, store) = super::open_world(&config)?;

    let _guard = super::lock_store(ctx)?;

    let mut instance = sekai_app::SekaiInstance::init(&store).await?;

    let (tx, rx) = watch::channel(None::<Progress>);
    let mut session = instance.world_mut(&mut world);
    let work = session.backup(BackupOptions::default(), scope, move |p| {
        let _ = tx.send(Some(p.into()));
    });

    let title = "backup";

    let (handle, (report, _)) = super::with_progress(ctx, title, rx, work).await?;

    handle
        .edit(ctx, CreateReply::new().embed(complete_embed(title, report)))
        .await?;

    Ok(())
}

fn complete_embed(title: &str, report: BackupReport) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(format!("{title} completed"))
        .field("new_blobs", report.new_blobs.to_string(), false)
        .field("chunks", report.chunks.to_string(), false)
        .field("snapshot", report.snapshot.raw().to_string(), false)
}
