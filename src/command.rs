use std::time::Duration;

use crate::{Context, config::Config};
use anyhow::{Result, bail};
use poise::{CreateReply, serenity_prelude::CreateEmbed};
use sekai_app::{BackupProgress, HostWorldTree, RollbackProgress};
use tokio::{
    sync::{MutexGuard, watch},
    time::MissedTickBehavior,
};

pub mod backup;
pub mod configure;
pub mod profile;
pub mod rollback;

fn open_world(config: &Config) -> Result<(HostWorldTree, String)> {
    config.validate()?;

    let Some(store) = config.store_path().to_str().map(str::to_owned) else {
        bail!("store path is not valid UTF-8");
    };

    Ok((HostWorldTree::new(config.world_path())?, store))
}

fn lock_store<'a>(ctx: Context<'a>) -> Result<MutexGuard<'a, ()>> {
    let Ok(guard) = ctx.data().store_lock.try_lock() else {
        bail!("another task is already running");
    };
    Ok(guard)
}

#[derive(Clone, Copy, Debug)]
struct Progress {
    files_done: usize,
    files_total: usize,
    chunks_done: usize,
}

impl From<BackupProgress> for Progress {
    fn from(progress: BackupProgress) -> Self {
        Self {
            files_done: progress.files_done,
            files_total: progress.files_total,
            chunks_done: progress.chunks_done,
        }
    }
}

impl From<RollbackProgress> for Progress {
    fn from(progress: RollbackProgress) -> Self {
        Self {
            files_done: progress.files_done,
            files_total: progress.files_total,
            chunks_done: progress.chunks_done,
        }
    }
}

fn progress_embed(title: &str, progress: Option<Progress>) -> CreateEmbed {
    match progress {
        Some(progress) => CreateEmbed::new()
            .title(title)
            .description("sekai")
            .field("files_done", progress.files_done.to_string(), false)
            .field("files_total", progress.files_total.to_string(), false)
            .field("chunks_done", progress.chunks_done.to_string(), false),
        None => CreateEmbed::default()
            .title(title)
            .description("preparing..."),
    }
}

async fn with_progress<'a, T>(
    ctx: Context<'a>,
    title: &str,
    mut rx: watch::Receiver<Option<Progress>>,
    work: impl Future<Output = Result<T, sekai_app::AppError>>,
) -> Result<(poise::ReplyHandle<'a>, T)> {
    tokio::pin!(work);

    let handle = ctx
        .send(CreateReply::default().embed(progress_embed(title, None)))
        .await?;

    let mut tick = tokio::time::interval(Duration::from_millis(500));
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let result = loop {
        tokio::select! {
            res = &mut work => break res,
            _ = tick.tick() => {
                if rx.has_changed().unwrap_or(false) {
                    let latest = *rx.borrow_and_update();
                    let _ = handle
                        .edit(ctx, CreateReply::default().embed(progress_embed(title, latest)))
                        .await;
                }
            }
        }
    };

    Ok((handle, result?))
}
