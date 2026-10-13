use anyhow::Result;

use crate::Context;
use crate::config::{Chunk, Config, Dimension, Profile};

#[poise::command(slash_command, owners_only, subcommands("add", "remove", "list"))]
pub async fn profile(_ctx: Context<'_>) -> Result<()> {
    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn add(
    ctx: Context<'_>,
    name: String,
    dim: Dimension,
    x1: i32,
    z1: i32,
    x2: i32,
    z2: i32,
) -> Result<()> {
    let chunk1 = Chunk::new(x1, z1);
    let chunk2 = Chunk::new(x2, z2);
    let profile = Profile::new(dim, chunk1, chunk2);

    let mut config = Config::load()?;

    let res = format!("added: {profile:?}");
    config.profiles.insert(name, profile);
    config.save()?;
    ctx.say(res).await?;

    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn remove(ctx: Context<'_>, name: String) -> Result<()> {
    let mut config = Config::load()?;

    if config.profiles.remove(&name).is_some() {
        config.save()?;
        ctx.say("Deleted {name:?} successfully").await?;
    } else {
        ctx.say("{name:?} is not found").await?;
    }

    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn list(ctx: Context<'_>) -> Result<()> {
    let config = Config::load()?;

    for profile in config.profiles {
        ctx.say(format!("{:?}", profile)).await?;
    }

    Ok(())
}
