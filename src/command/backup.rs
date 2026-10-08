use anyhow::Result;
use std::path::Path;

pub async fn run(world_path: impl AsRef<Path>, store_path: &str) -> Result<()> {
    let mut instance = sekai_app::SekaiInstance::init(store_path).await?;
    let (report, timings) = instance
        .world_mut(world_path)
        .backup(
            sekai_app::BackupOptions::default(),
            sekai_app::Scope::World,
            |_| {},
        )
        .await?;
    println!("{:?}, {:?}", report, timings);
    Ok(())
}
