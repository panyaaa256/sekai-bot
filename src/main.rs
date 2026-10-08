use anyhow::Result;
use config::Config;
use poise::serenity_prelude as serenity;
mod command;

struct Data {} // User data, which is stored and accessible in all command invocations
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

#[poise::command(slash_command)]
async fn config(
    ctx: Context<'_>,
    #[description = "Select action"]
    #[choices("world_path", "store_path")]
    key: &'static str,
    value: String,
) -> Result<()> {
    let setting = Config::builder()
        .add_source(config::File::with_name("settings"))
        .build()?;
    let world_path = setting.get_string(key)?;
    let res = format!("{:?}, {}", world_path, value);
    ctx.say(res).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn backup(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;
    let world_path = dotenvy::var("WORLD_PATH")?;
    let store_path = dotenvy::var("SEKAI_PATH")?;
    command::backup::run(&world_path, &store_path).await?;
    ctx.say("Backup completed").await?;
    println!("{}", ctx.author());
    Ok(())
}

#[poise::command(slash_command)]
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
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
