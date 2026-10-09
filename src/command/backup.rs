use anyhow::{Result, bail};
use sekai_app::BackupReport;
use std::path::Path;

pub async fn run(world_path: &Path, store_path: &Path) -> Result<BackupReport> {
    let Some(store) = store_path.to_str() else {
        bail!("store path is not valid UTF-8: {:?}", store_path);
    };

    let mut instance = sekai_app::SekaiInstance::init(store).await?;
    let mut world = sekai_app::HostWorldTree::new(world_path)?;

    let (report, _) = instance
        .world_mut(&mut world)
        .backup(
            sekai_app::BackupOptions::default(),
            sekai_app::Scope::World,
            |_| {},
        )
        .await?;

    Ok(report)
}
