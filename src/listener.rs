use crate::git::*;

use std::collections::HashSet;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result};
use notify::{RecursiveMode, Watcher};
use notify::event::EventKind;
use std::time::{Duration, Instant};
use std::sync::mpsc;

const GIT_PUSH_INTERVAL: u64 = 1;
const MAX_PUSH_DELAY: u64 = 15;

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
            remote_poll_interval: 2,
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

        let (tx, rx) = mpsc::channel::<PathBuf>();

        let mut first_change: Option<Instant> = None;
        let mut last_change: Option<Instant> = None;

        let mut push_pending: HashSet<PathBuf> = HashSet::new();

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

                                    println!("Share changed: {}", share_path.display());

                                    let _ = tx.send(share_path.clone());
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
            let timeout = match (first_change, last_change) {
                (Some(first), Some(last)) => {
                    let quiet_left = Duration::from_secs(GIT_PUSH_INTERVAL).saturating_sub(last.elapsed());
                    let max_left = Duration::from_secs(MAX_PUSH_DELAY).saturating_sub(first.elapsed());

                    quiet_left.min(max_left)
                },
                _ => Duration::from_secs(self.config.remote_poll_interval),
            };

            match rx.recv_timeout(timeout) {
                Ok(share_path) => {
                    let now = Instant::now();

                    if first_change.is_none() {
                        first_change = Some(now);
                    }

                    last_change = Some(now);

                    push_pending.insert(share_path);
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(share_path) = Some(push_pending.clone()) {
                        for p in share_path.clone().into_iter() {
                            let git = GitManager {
                                repo_dir: p.clone(),
                            };

                            println!("Pushing share {}", p.display());

                            if let Err(error) = git.push("share update") {
                                eprintln!("Push failed: {error:#}");
                            }
                        }
                    }

                    for p in &self.config.share_paths {
                        let git = GitManager {
                            repo_dir: p.to_path_buf(),
                        };

                        if !git.origin_matches_local()? {
                            println!("Syncing share {} to origin", p.display());
                            git.pull()?;
                        }
                    }

                    first_change = None;
                    last_change = None;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => (),
            }
        }
    }
}