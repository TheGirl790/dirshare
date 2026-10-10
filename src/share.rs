use crate::git::*;
use crate::listener::ListenerConfig;
use crate::manifest::*;

use std::fs;
use std::path::PathBuf;

use anyhow::{Result, bail};

pub fn new_share(dir: &PathBuf, config_path: &PathBuf, name: String, remote: String) -> Result<()> {
    let mut share_dir = dir.clone();
    share_dir.push(&name);

    let git = GitManager{
        repo_dir: share_dir.clone(),
    };

    println!("Creating new share at {}", share_dir.display());
    println!("Using remote repository at {}", remote);

    if share_dir.exists() {
        let is_empty = fs::read_dir(&share_dir)?.next().is_none();

        if !is_empty {
            bail!("New share directory exists and is not empty.");
        }
    }

    println!("\nCloning remote repository into new share...");
    git.clone(&remote)?;

    let manifest_path = share_dir.join(".dirshare").join("share.toml");

    if !manifest_path.is_file() {
        println!("Setting up new share...");

        let manifest = ShareManifest {
            name: name.clone(),
            remote_url: remote,
            status: ShareStatus::Active,
        };

        println!("Writing new TOML manifest...");
        let new_text = toml::to_string_pretty(&manifest)?;
        fs::create_dir_all(manifest_path.parent().unwrap())?;
        fs::write(manifest_path, new_text)?;

        println!("Pushing changes...");
        git.push("dirshare setup")?;

        println!("Adding share to listener config...");
        let mut config = ListenerConfig::load(&config_path)?;
        config.share_paths.push(share_dir);
        config.write(&config_path)?;

        println!("Successfully created share '{}'!", name);

    } else {
        let contents = fs::read_to_string(manifest_path)?;
        let _: ShareManifest = toml::from_str(&contents)?;

        eprintln!("Remote is already a share, aborting...\nIf you want to join an existing share, try using `dirshare join`");
        fs::remove_dir_all(share_dir)?;
    }

    Ok(())
}

pub fn join_share(dir: &PathBuf, config_path: &PathBuf, remote: String) -> Result<()> {
    let mut share_dir = dir.clone();
    share_dir.push("temp");

    let mut idx = 0;
    while share_dir.exists() {
        share_dir = share_dir.parent().ok_or_else(|| anyhow::anyhow!("Path has no parent directory"))?.to_path_buf();
        share_dir.push(format!("temp{idx}"));

        idx += 1;
    }

    let git = GitManager{
        repo_dir: share_dir.clone(),
    };

    println!("Joining existing share at {}", remote);

    println!("\nCloning remote repository...");
    git.clone(&remote)?;

    let manifest_path = share_dir.join(".dirshare").join("share.toml");

    if manifest_path.exists() && manifest_path.is_file() {
        println!("Configuring share...");
        let mut manifest = ShareManifest::load(&manifest_path)?;

        manifest.status = ShareStatus::Active;

        let parent = share_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Share directory has no parent"))?;

        let new_share_dir = parent.join(&manifest.name);

        fs::rename(&share_dir, &new_share_dir)?;

        println!("Adding share to listener config...");
        let mut config = ListenerConfig::load(&config_path)?;
        config.share_paths.push(new_share_dir);
        config.write(&config_path)?;

        println!("Successfully joined share '{}'!", manifest.name);
    } else {
        eprintln!("Remote is not a share, aborting...\nIf you want to create a new share, try using `dirshare new`");
        fs::remove_dir_all(share_dir)?;
    }

    Ok(())
}