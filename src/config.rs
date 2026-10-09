use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::{fs, io::ErrorKind};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Config {
    pub root_path: PathBuf,
    pub profile: Vec<Profile>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    dim: Dimension,
    area: (Chunk, Chunk),
}

#[derive(Serialize, Deserialize, Debug)]
enum Dimension {
    Overworld,
    Nether,
    End,
}

#[derive(Serialize, Deserialize, Debug)]
struct Chunk {
    x: i32,
    z: i32,
}

impl Config {
    fn from_json(file: File) -> Result<Config> {
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let data = serde_json::to_string(self)?;
        fs::write(Self::config_path(), data)
            .with_context(|| format!("failed to write {}", Self::config_path().display()))?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        match File::open(Self::config_path()) {
            Ok(file) => Self::from_json(file),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    fn config_path() -> PathBuf {
        dotenvy::var("CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("config.json"))
    }
}
