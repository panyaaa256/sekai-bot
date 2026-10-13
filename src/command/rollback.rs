use anyhow::Result;
use poise::{CreateReply, serenity_prelude::CreateEmbed};
use sekai_app::{RollbackOptions, RollbackReport, SnapshotId};
use tokio::sync::watch;

use crate::{Context, command::Progress, config::Config};

#[poise::command(slash_command, owners_only)]
pub async fn rollback(ctx: Context<'_>, id: u64, name: Option<String>) -> Result<()> {
    ctx.defer().await?;

    let config = Config::load()?;
    let scope = config.resolve_scope(name)?;
    let (mut world, store) = super::open_world(&config)?;

    let _guard = super::lock_store(ctx)?;

    let mut instance = sekai_app::SekaiInstance::open(&store).await?;

    let (tx, rx) = watch::channel(None::<Progress>);
    let mut session = instance.world_mut(&mut world);
    let work = session.rollback(
        SnapshotId(id),
        RollbackOptions::default(),
        scope,
        move |p| {
            let _ = tx.send(Some(p.into()));
        },
    );

    let title = "rollback";

    let (handle, (report, _)) = super::with_progress(ctx, title, rx, work).await?;

    handle
        .edit(ctx, CreateReply::new().embed(complete_embed(title, report)))
        .await?;

    Ok(())
}

fn complete_embed(title: &str, report: RollbackReport) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(format!("{title} completed"))
        .field("chunks_restored", report.chunks_restored.to_string(), false)
        .field("files_deleted", report.files_deleted.to_string(), false)
        .field("files_written", report.files_written.to_string(), false)
}
