use crate::config::Config;
use anyhow::{Result, bail};
use sekai_app::HostWorldTree;

pub mod backup;
pub mod configure;
pub mod profile;
pub mod rollback;

fn open_world() -> Result<(HostWorldTree, String)> {
    let config = Config::load()?;
    config.validate()?;

    let Some(store) = config.store_path().to_str().map(str::to_owned) else {
        bail!("store path is not valid UTF-8");
    };

    Ok((HostWorldTree::new(config.world_path())?, store))
}
