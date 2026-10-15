use anyhow::Result;
use poise::ChoiceParameter;
use poise::CreateReply;
use poise::serenity_prelude::Colour;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateEmbedAuthor;
use sekai_app::Rect;

use crate::Context;
use crate::config::{Chunk, Config, Dimension, Profile};

#[poise::command(
    slash_command,
    owners_only,
    subcommands("add", "remove", "info", "edit")
)]
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
    let mut config = Config::load()?;

    if config.profiles.contains_key(&name) {
        let reply = error_reply(format!("Profile: `{name}` already exists"));

        ctx.send(reply).await?;

        return Ok(());
    };

    let chunk1 = Chunk::new(x1, z1);
    let chunk2 = Chunk::new(x2, z2);
    let profile = Profile::new(dim, chunk1, chunk2);

    let reply = summary_reply(&name, &profile, Action::Add);

    config.profiles.insert(name, profile);
    config.save()?;

    ctx.send(reply).await?;
    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn remove(
    ctx: Context<'_>,
    #[autocomplete = "super::autocomplete_profile"] name: String,
) -> Result<()> {
    let mut config = Config::load()?;

    let Some(profile) = config.profiles.remove(&name) else {
        let reply = error_reply(format!("Profile: `{name}` not found"));

        ctx.send(reply).await?;

        return Ok(());
    };

    config.save()?;
    ctx.send(summary_reply(&name, &profile, Action::Remove))
        .await?;

    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn edit(
    ctx: Context<'_>,
    #[autocomplete = "super::autocomplete_profile"] name: String,
    dim: Dimension,
    x1: i32,
    z1: i32,
    x2: i32,
    z2: i32,
) -> Result<()> {
    let mut config = Config::load()?;

    if !config.profiles.contains_key(&name) {
        let reply = error_reply(format!("Profile: `{name}` not found"));

        ctx.send(reply).await?;

        return Ok(());
    }

    let chunk1 = Chunk::new(x1, z1);
    let chunk2 = Chunk::new(x2, z2);
    let profile = Profile::new(dim, chunk1, chunk2);

    let reply = summary_reply(&name, &profile, Action::Edit);

    config.profiles.insert(name, profile);
    config.save()?;

    ctx.send(reply).await?;

    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn info(
    ctx: Context<'_>,
    #[autocomplete = "super::autocomplete_profile"] name: Option<String>,
) -> Result<()> {
    let config = Config::load()?;

    let reply = if config.profiles.is_empty() {
        error_reply("No profiles registered")
    } else if let Some(key) = name {
        match config.profiles.get(&key) {
            Some(profile) => summary_reply(&key, profile, Action::Info),
            None => error_reply(format!("Profile: `{key}` not found")),
        }
    } else {
        let mut rows = vec![("name", "dim", "center".to_string(), "size".to_string())];
        rows.extend(config.profiles.iter().map(|(name, profile)| {
            (
                name.as_str(),
                profile.dim.name(),
                center_text(profile),
                size_text(profile),
            )
        }));
        let name_width = rows
            .iter()
            .map(|(name, _, _, _)| name.chars().count())
            .max()
            .unwrap_or(0);
        let dim_width = rows
            .iter()
            .map(|(_, dim, _, _)| dim.chars().count())
            .max()
            .unwrap_or(0);
        let center_width = rows
            .iter()
            .map(|(_, _, center, _)| center.chars().count())
            .max()
            .unwrap_or(0);
        let size_width = rows
            .iter()
            .map(|(_, _, _, size)| size.chars().count())
            .max()
            .unwrap_or(0);
        let mut lines = rows
            .iter()
            .map(|(name, dim, center, size)| {
                format!("{name:<name_width$}  {dim:<dim_width$}  {center:<center_width$}  {size}")
            })
            .collect::<Vec<_>>();

        lines.insert(
            1,
            format!(
                "{}  {}  {}  {}",
                "-".repeat(name_width),
                "-".repeat(dim_width),
                "-".repeat(center_width),
                "-".repeat(size_width)
            ),
        );
        let description = format!("```\n{}\n```", lines.join("\n"));
        let count = config.profiles.len();
        let title = format!("{} profile{}", count, if count == 1 { "" } else { "s" });

        CreateReply::new().embed(
            CreateEmbed::new()
                .color(Colour::FOOYOO)
                .author(CreateEmbedAuthor::new("Profile List"))
                .title(title)
                .description(description),
        )
    };

    ctx.send(reply).await?;

    Ok(())
}

enum Action {
    Add,
    Remove,
    Edit,
    Info,
}

impl Action {
    fn title(&self) -> &'static str {
        match self {
            Self::Add => "Added",
            Self::Remove => "Removed",
            Self::Edit => "Updated",
            Self::Info => "Info",
        }
    }
}

fn summary_reply(name: &str, profile: &Profile, action: Action) -> CreateReply {
    CreateReply::new().embed(
        CreateEmbed::new()
            .color(Colour::FOOYOO)
            .author(CreateEmbedAuthor::new(format!(
                "Profile {}",
                action.title()
            )))
            .title(name.to_string())
            .field("dim", profile.dim.name(), true)
            .field("center", center_text(profile), true)
            .field("size", size_text(profile), true)
            .field("range", area_text(profile), false),
    )
}

fn error_reply(description: impl Into<String>) -> CreateReply {
    CreateReply::new().ephemeral(true).embed(
        CreateEmbed::new()
            .color(Colour::RED)
            .author(CreateEmbedAuthor::new("Profile Error"))
            .description(description),
    )
}

fn area_text(profile: &Profile) -> String {
    let Rect { x0, z0, x1, z1 } = profile.rect();

    if (x0, z0) == (x1, z1) {
        format!("({x0}, {z0})")
    } else {
        format!("({x0}, {z0}) -> ({x1}, {z1})")
    }
}

fn center_text(profile: &Profile) -> String {
    let Rect { x0, z0, x1, z1 } = profile.rect();
    let cx = (x0 as f64 + x1 as f64) / 2.0;
    let cz = (z0 as f64 + z1 as f64) / 2.0;

    format!("({cx}, {cz})")
}

fn size_text(profile: &Profile) -> String {
    let Rect { x0, z0, x1, z1 } = profile.rect();

    format!("{} x {}", x1 - x0 + 1, z1 - z0 + 1)
}
