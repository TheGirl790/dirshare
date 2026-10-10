use crate::git::*;

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result};
use notify::{RecursiveMode, Watcher};
use notify::event::EventKind;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ListenerConfig {
    pub share_paths: Vec<PathBuf>,
    pub active: bool,
    pub remote_poll_interval: u64,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            share_paths: Vec::new(),
            active: true,
            remote_poll_interval: 5,
        }
    }
}

impl ListenerConfig {
    pub fn load(file: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(file)?;
        let loaded: ListenerConfig = toml::from_str(&contents)?;

        Ok(loaded)
    }

    pub fn write(&self, file: &PathBuf) -> Result<()> {
        let new_text = toml::to_string_pretty(&self)?;
        fs::create_dir_all(file.parent().unwrap())?;
        fs::write(&file, new_text)?;
        
        Ok(())
    }
}

pub struct ShareListener {
    config: ListenerConfig,
}

impl ShareListener {
    pub fn new(config_path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(&config_path)?;
        let config: ListenerConfig = toml::from_str(&contents)?;

        Ok(ShareListener {
            config
        })
    }

    pub fn start(&self) -> Result<()> {
        let paths = self.config.share_paths.clone();

        println!("Initialising listener...");
        let mut watcher = notify::recommended_watcher(move |result: std::prelude::v1::Result<notify::Event, notify::Error>| {
            match result {
                Ok(event) => {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            for event_path in &event.paths {
                                if let Some(share_path) = paths
                                    .iter()
                                    .find(|share| event_path.starts_with(share))
                                {
                                    let git_dir = share_path.join(".git");

                                    if event_path.starts_with(&git_dir) {
                                        continue;
                                    }

                                    println!("Changed share: {}", share_path.display());
                                    println!("Changed path: {}", event_path.display());

                                    println!("Pushing changes to remote share repository...");

                                    let git = GitManager {
                                        repo_dir: share_path.clone(),
                                    };

                                    let _ = git.push("share update");
                                }
                            }
                        }
                        _ => (),
                    }
                },
                Err(error) => eprintln!("{error}"),
            }
        })?;

        println!("Adding share paths to listener...");
        for path in &self.config.share_paths {
            println!("listening at {}", path.display());
            if path.exists() {
                watcher.watch(&path.as_path(), RecursiveMode::Recursive)?;
            } else {
                println!("Warning: failed to listen path {}: does not exist", path.display());
            }
        }

        loop {
            std::thread::park();
        }
    }
}