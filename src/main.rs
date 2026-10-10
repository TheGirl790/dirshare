use clap::{Parser, Subcommand};
use std::{env};
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Subcommand, Debug)]
enum Command {
    New { share_name: String, remote: String },
    Join { remote: String },
    Status,
    Start,
    Stop,
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command
}

#[derive (Debug, Deserialize, Serialize)]
struct ShareManifest {
    name: String,
    remote_url: String,
}

fn clone_repo(url: &str, dest: &Path) -> Result<()> {
    let output = ProcessCommand::new("git")
        .arg("clone")
        .arg(url)
        .arg(dest)
        .output()
        .context("Could not launch Git. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Git clone failed: {stderr}");
    }

    Ok(())
}

fn git_push(msg: &str, repo_dir: &str) -> Result<()> {
    let output = ProcessCommand::new("git")
        .arg("add")
        .arg(".")
        .current_dir(repo_dir)
        .output()
        .context("Could not launch Git. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Git add failed: {stderr}");
    }

    let output = ProcessCommand::new("git")
        .arg("commit")
        .arg("-m")
        .arg(msg)
        .current_dir(repo_dir)
        .output()
        .context("Could not launch Git. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Git commit failed: {stderr}");
    }

    let output = ProcessCommand::new("git")
        .arg("push")
        .arg("origin")
        .arg("main")
        .current_dir(repo_dir)
        .output()
        .context("Could not launch Git. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Git push failed: {stderr}");
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::New { share_name, remote } => {
            let mut share_dir = env::current_dir().context("Failed to resolve working directory.")?;
            share_dir.push(&share_name);

            println!("Creating new share at {}", share_dir.display());
            println!("Using remote repository at {}", remote);

            if share_dir.exists() {
                let is_empty = fs::read_dir(&share_dir)?.next().is_none();

                if !is_empty {
                    bail!("New share directory exists is not empty.");
                }
            }

            println!("\nCloning remote repository into new share...");
            clone_repo(&remote, &share_dir)?;

            let manifest_path = share_dir.join(".dirshare").join("share.toml");

            if !manifest_path.is_file() {
                println!("Setting up new share...");

                let manifest = ShareManifest {
                    name: share_name,
                    remote_url: remote,
                };

                println!("Writing new TOML manifest...");
                let new_text = toml::to_string_pretty(&manifest)?;
                fs::create_dir_all(manifest_path.parent().unwrap())?;
                fs::write(manifest_path, new_text)?;

                println!("Pushing changes...");
                git_push("dirshare setup", &share_dir.to_string_lossy())?;

                println!("Done!");

            } else {
                let contents = fs::read_to_string(manifest_path)?;
                let _: ShareManifest = toml::from_str(&contents)?;

                println!("Remote is already a share, aborting...\nIf you want to join an existing share, try using `dirshare join`");

                fs::remove_dir_all(share_dir)?;
            }
        },
        Command::Join { remote } => {
            let mut share_dir = env::current_dir().context("Failed to resolve working directory.")?;
            share_dir.push("temp");

            let mut idx = 0;
            while share_dir.exists() {
                share_dir = share_dir.parent().ok_or_else(|| anyhow::anyhow!("Path has no parent directory"))?.to_path_buf();
                share_dir.push(format!("temp{idx}"));

                idx += 1;
            }

            println!("Joining existing share at {}", remote);

            println!("\nCloning remote repository...");
            clone_repo(&remote, &share_dir)?;

            let manifest_path = share_dir.join(".dirshare").join("share.toml");

            if manifest_path.exists() && manifest_path.is_file() {
                println!("Configuring share...");
                let contents = fs::read_to_string(manifest_path)?;
                let manifest: ShareManifest = toml::from_str(&contents)?;

                let parent = share_dir
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("Share directory has no parent"))?;

                let new_share_dir = parent.join(&manifest.name);

                fs::rename(&share_dir, &new_share_dir)?;
            } else {
                println!("Remote is not a share, aborting...\nIf you want to create a new share, try using `dirshare new`");

                fs::remove_dir_all(share_dir)?;
            }
        }
        _ => (),
    };

    Ok(())
}