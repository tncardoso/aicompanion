use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub thresholds: Thresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub coupling: u32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            cyclomatic: 10,
            cognitive: 15,
            coupling: 5,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            thresholds: Thresholds::default(),
        }
    }
}

pub fn load(repo_root: &Path) -> Result<Config> {
    let config_path = repo_root.join(".aicompanion.toml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}
