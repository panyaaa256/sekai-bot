use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::{fs, io::ErrorKind};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Config {
    pub root_path: PathBuf,
    pub profiles: Vec<Profile>,
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
        let path = Self::config_path();
        fs::write(&path, data).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        match File::open(Self::config_path()) {
            Ok(file) => Self::from_json(file),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                File::create_new(Self::config_path())?;
                Ok(Self::default())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn config_path() -> PathBuf {
        dotenvy::var("CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("config.json"))
    }

    pub fn world_path(&self) -> PathBuf {
        self.root_path.join("world")
    }

    pub fn store_path(&self) -> PathBuf {
        self.root_path.join("sekai")
    }

    pub fn validate(&self) -> Result<()> {
        if !self.root_path.is_absolute() {
            bail!("root path must be absolute: {:?}", self.root_path);
        }
        if !self.world_path().is_dir() {
            bail!("world directory not found: {}", self.world_path().display());
        }
        Ok(())
    }
}
