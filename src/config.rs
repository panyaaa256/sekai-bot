use anyhow::{Context, Result, bail};
use sekai_app::{Area, Rect, Scope};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::{fs, io::ErrorKind};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Config {
    pub root_path: PathBuf,
    pub profiles: BTreeMap<String, Profile>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub dim: Dimension,
    pub area: (Chunk, Chunk),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, poise::ChoiceParameter)]
pub enum Dimension {
    #[name = "overworld"]
    Overworld,
    #[name = "the_nether"]
    Nether,
    #[name = "the_end"]
    End,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chunk(i32, i32);

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
                let config = Self::default();
                Self::save(&config)?;
                Ok(config)
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

    pub fn resolve_scope(&self, name: Option<String>) -> Result<Scope> {
        match name {
            Some(key) => {
                let Some(profile) = self.profiles.get(&key) else {
                    bail!("profile not found: {key}");
                };
                Ok(profile.scope())
            }
            None => Ok(Scope::World),
        }
    }
}

impl Profile {
    pub fn new(dim: Dimension, chunk1: Chunk, chunk2: Chunk) -> Self {
        Profile {
            dim,
            area: (chunk1, chunk2),
        }
    }

    pub fn scope(&self) -> Scope {
        let (a, b) = &self.area;
        Scope::Select {
            kinds: (Scope::all_kinds()),
            areas: vec![(self.dim.into(), Area::Rect(Rect::new(a.0, a.1, b.0, b.1)))],
        }
    }
}

impl From<Dimension> for sekai_app::Dimension {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Overworld => Self::OVERWORLD,
            Dimension::Nether => Self::NETHER,
            Dimension::End => Self::END,
        }
    }
}

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        Chunk(x, z)
    }
}
