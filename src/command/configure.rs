use anyhow::Result;
use std::path::PathBuf;

use crate::Context;
use crate::config::Config;

#[derive(poise::ChoiceParameter, Debug)]
enum Key {
    RootPath,
    Profile,
}

#[poise::command(slash_command, owners_only, rename = "config")]
pub async fn configure(ctx: Context<'_>, key: Key, value: String) -> Result<()> {
    let mut config = Config::load()?;

    match key {
        Key::RootPath => {
            let root = PathBuf::from(value);
            config.root_path = root;
            config.validate()?;
        }
        Key::Profile => {
            // TODO: add custom area
            ctx.say("not implemented").await?;
            return Ok(());
        }
    }

    config.save()?;
    let res = config.root_path.display().to_string();
    ctx.say(res).await?;

    Ok(())
}
