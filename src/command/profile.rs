use anyhow::Result;
use poise::ChoiceParameter;
use poise::CreateReply;
use poise::serenity_prelude::Colour;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateEmbedAuthor;
use sekai_app::Rect;

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

    let embed = summary(&name, &profile, Action::Add);

    config.profiles.insert(name, profile);
    config.save()?;

    ctx.send(CreateReply::new().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn remove(
    ctx: Context<'_>,
    #[autocomplete = "super::autocomplete_profile"] name: String,
) -> Result<()> {
    let mut config = Config::load()?;

    let embed;
    let ephemeral;

    if let Some(profile) = config.profiles.get(&name) {
        embed = summary(&name, profile, Action::Remove);
        config.profiles.remove(&name);
        ephemeral = false;
    } else {
        embed = CreateEmbed::new()
            .author(CreateEmbedAuthor::new("Profile Error"))
            .color(Colour::RED)
            .description(format!("profile: `{name}` not found"));
        ephemeral = true;
    }

    ctx.send(CreateReply::new().embed(embed).ephemeral(ephemeral))
        .await?;

    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn list(ctx: Context<'_>) -> Result<()> {
    let config = Config::load()?;

    for profile in config.profiles {
        ctx.say(format!("{profile:?}")).await?;
    }

    Ok(())
}

enum Action {
    Add,
    Remove,
}

impl Action {
    fn title(&self) -> &'static str {
        match self {
            Self::Add => "Added",
            Self::Remove => "Removed",
        }
    }
}

fn summary(name: &str, profile: &Profile, action: Action) -> CreateEmbed {
    let Rect { x0, z0, x1, z1 } = profile.rect();

    let area_value = if (x0, z0) == (x1, z1) {
        format!("({x0}, {z0})")
    } else {
        format!("({x0}, {z0}) -> ({x1}, {z1})")
    };

    CreateEmbed::new()
        .author(CreateEmbedAuthor::new(format!(
            "Profile {}",
            action.title()
        )))
        .title(name.to_string())
        .color(Colour::FOOYOO)
        .field("dim", profile.dim.name(), true)
        .field("chunk", area_value, true)
}
