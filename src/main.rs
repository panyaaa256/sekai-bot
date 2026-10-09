use anyhow::Result;
use poise::serenity_prelude as serenity;
use std::path::PathBuf;
use tokio::sync::Mutex;

mod command;
mod config;

type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

struct Data {
    store_lock: Mutex<()>,
}

#[derive(poise::ChoiceParameter, Debug)]
enum Key {
    RootPath,
    Profile,
}

#[poise::command(slash_command, owners_only)]
async fn config(ctx: Context<'_>, key: Key, value: String) -> Result<()> {
    let mut config = config::Config::load()?;

    match key {
        Key::RootPath => {
            let root = PathBuf::from(value);
            config.root_path = root;
        }
        Key::Profile => {
            //add custom area
            ctx.say("not implemented").await?;
            return Ok(());
        }
    }

    config.save()?;
    let res = format!("{}", config.root_path.display());
    ctx.say(res).await?;

    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn backup(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;

    let config = config::Config::load()?;
    let world_path = config.root_path.join("world");
    let store_path = config.root_path.join("sekai");

    if config.root_path.as_os_str().is_empty() || !world_path.exists() {
        ctx.say("set valid root path").await?;
        return Ok(());
    }

    let Ok(_guard) = ctx.data().store_lock.try_lock() else {
        ctx.say("backup is already running").await?;
        return Ok(());
    };

    let report = command::backup::run(&world_path, &store_path).await?;
    let res = format!("new blobs: {}", report.new_blobs);

    ctx.say("Backup completed").await?;
    ctx.say(res).await?;

    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn rollback(ctx: Context<'_>) -> Result<()> {
    println!("{}", ctx.author());
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = dotenvy::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![backup(), rollback(), config()],
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
