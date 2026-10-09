use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;

use crate::command::{backup, configure, rollback};

mod command;
mod config;

type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

struct Data {
    store_lock: Mutex<()>,
}

#[tokio::main]
async fn main() {
    let token = dotenvy::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                backup::backup(),
                rollback::rollback(),
                configure::configure(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    store_lock: Mutex::new(()),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
