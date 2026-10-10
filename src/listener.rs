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
use std::sync::{Arc, RwLock};

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

enum ListenerMsg {
    ShareChanged{ path: PathBuf },
    ConfigChanged,
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

    pub fn start(&mut self, config_dir: PathBuf) -> Result<()> {
        let (tx, rx) = mpsc::channel::<ListenerMsg>();

        let mut watched_paths: HashSet<PathBuf> = HashSet::new();

        let mut first_change: Option<Instant> = None;
        let mut last_change: Option<Instant> = None;

        let mut push_pending: HashSet<PathBuf> = HashSet::new();

        println!("Initialising listener...");
        let callback_config_dir = config_dir.clone();
        let paths: Arc<RwLock<Vec<PathBuf>>> = Arc::new(RwLock::new(self.config.share_paths.clone()));
        let watcher_paths = Arc::clone(&paths);
        let mut watcher = notify::recommended_watcher(move |result: std::prelude::v1::Result<notify::Event, notify::Error>| {
            match result {
                Ok(event) => {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            for event_path in &event.paths {
                                if event_path.clone() == callback_config_dir {
                                    let _ = tx.send(ListenerMsg::ConfigChanged);
                                }

                                let callback_paths = watcher_paths.read().unwrap();

                                if let Some(share_path) = callback_paths
                                    .iter()
                                    .find(|share| event_path.starts_with(share))
                                {
                                    let git_dir = share_path.join(".git");

                                    if event_path.starts_with(&git_dir) {
                                        continue;
                                    }

                                    println!("Share changed: {}", share_path.display());

                                    let _ = tx.send(ListenerMsg::ShareChanged { path: share_path.clone() });
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
                watched_paths.insert(path.clone());
            } else {
                println!("Warning: path {} does not exist", path.display());

                let mut file_config = ListenerConfig::load(&config_dir)?;
                if file_config.share_paths.contains(path) {
                    println!("Removing non-existant path {} from config file", path.display());
                    file_config.share_paths.retain(|item| item != path);
                }

                file_config.write(&config_dir)?;
            }
        }

        println!("Listening at config directory: {}", config_dir.parent().unwrap().display());
        watcher.watch(config_dir.parent().unwrap(), RecursiveMode::NonRecursive)?;

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
                Ok(msg) => {
                    match msg {
                        ListenerMsg::ShareChanged { path } => {
                            let now = Instant::now();

                            if first_change.is_none() {
                                first_change = Some(now);
                            }

                            last_change = Some(now);

                            push_pending.insert(path);
                        },
                        ListenerMsg::ConfigChanged => {
                            println!("Listener config changed! updating config...");

                            let new_config = ListenerConfig::load(&config_dir)?;
                            let new_paths = new_config.share_paths;

                            let mut config_paths = paths.write().unwrap();

                            let old_paths = config_paths.clone();

                            *config_paths = new_paths.clone();

                            let mut all_paths: HashSet<PathBuf> = HashSet::new();

                            for p in &old_paths {
                                all_paths.insert(p.clone());
                            }

                            for p in &new_paths {
                                all_paths.insert(p.clone());
                            }

                            for p in all_paths.iter() {
                                if old_paths.contains(p) && !new_paths.contains(p) && watched_paths.contains(p) {
                                    watcher.unwatch(p)?;
                                    watched_paths.remove(p);
                                }

                                if !old_paths.contains(p) && new_paths.contains(p) {
                                    println!("listening at {}", p.display());
                                    if p.exists() {
                                        watcher.watch(&p.as_path(), RecursiveMode::Recursive)?;
                                        watched_paths.insert(p.clone());
                                    } else {
                                        println!("Warning: failed to listen path {}: does not exist", p.display());
                                    }
                                }
                            }
                        }
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(_) = Some(push_pending.clone()) {
                        let pending = std::mem::take(&mut push_pending);

                        for p in pending {
                            let git = GitManager { repo_dir: p.clone() };

                            println!("Pushing share {}", p.display());

                            if let Err(error) = git.push("share update") {
                                eprintln!("Push failed: {error:#}");
                            }
                        }
                    }

                    for p in &self.config.share_paths {
                        if !p.is_dir() {
                            continue;
                        }

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