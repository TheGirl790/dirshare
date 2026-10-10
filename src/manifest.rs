use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Ok, Result};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub enum ShareStatus {
    Active,
    Paused,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ShareManifest {
    pub name: String,
    pub remote_url: String,
    pub status: ShareStatus,
}

impl ShareManifest {
    pub fn load(file: &Path) -> Result<Self> {
        let contents = fs::read_to_string(file)?;
        let loaded: ShareManifest = toml::from_str(&contents)?;

        Ok(loaded)
    }
}