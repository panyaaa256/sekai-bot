use anyhow::Result;
use poise::{
    CreateReply,
    serenity_prelude::{Colour, CreateEmbed, CreateEmbedAuthor},
};

use crate::{Context, config::Config};

#[poise::command(slash_command, owners_only, subcommands("create", "remove"))]
pub async fn tag(_ctx: Context<'_>) -> Result<()> {
    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn create(ctx: Context<'_>, snapshot: String, name: String) -> Result<()> {
    let name = sekai_app::TagName::parse(&name)?;
    let config = Config::load()?;
    let (_, store) = super::open_world(&config)?;
    let mut instance = sekai_app::SekaiInstance::open(&store).await?;
    let id = instance.resolve_snapshot_ref(&snapshot).await?;
    instance.create_tag(&name, id, false).await?;
    ctx.send(tag_reply(Action::Create, name.to_string(), id.raw()))
        .await?;

    Ok(())
}

#[poise::command(slash_command, owners_only)]
async fn remove(ctx: Context<'_>, name: String) -> Result<()> {
    let name = sekai_app::TagName::parse(&name)?;
    let config = Config::load()?;
    let (_, store) = super::open_world(&config)?;
    let mut instance = sekai_app::SekaiInstance::open(&store).await?;
    let id = instance
        .resolve_snapshot_ref(format!("@{name}").as_str())
        .await?;
    instance.delete_tag(&name).await?;
    ctx.send(tag_reply(Action::Remove, name.to_string(), id.raw()))
        .await?;

    Ok(())
}

enum Action {
    Create,
    Remove,
}

impl Action {
    fn title(&self) -> &'static str {
        match self {
            Self::Create => "Created",
            Self::Remove => "Removed",
        }
    }
}

fn tag_reply(action: Action, name: String, id: u64) -> CreateReply {
    CreateReply::new().embed(
        CreateEmbed::new()
            .color(Colour::FOOYOO)
            .author(CreateEmbedAuthor::new(format!("Tag {}", action.title())))
            .field("name", name.to_string(), true)
            .field("id", id.to_string(), true),
    )
}
