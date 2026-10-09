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

fn git_push(msg: &str) -> Result<()> {
    let output = ProcessCommand::new("git")
        .arg("add")
        .arg(".")
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

            println!("\nCloning remote repository into new share...");
            clone_repo(&remote, &share_dir)?;
            println!("Clone finished!");

            let manifest_path = share_dir.join(".dirshare").join("share.toml");

            if !manifest_path.exists() && !manifest_path.is_file() {
                println!("\nSetting up new share...");

                let manifest = ShareManifest {
                    name: share_name,
                    remote_url: remote,
                };

                println!("Writing new TOML manifest...");
                let new_text = toml::to_string_pretty(&manifest)?;
                fs::create_dir_all(manifest_path.parent().unwrap())?;
                fs::write(manifest_path, new_text)?;

                println!("Finished writing manifest.\nPushing changes...");
                git_push("dirshare setup")?;

            } else {
                if manifest_path.is_file() {
                    let contents = fs::read_to_string(manifest_path)?;
                    let _: ShareManifest = toml::from_str(&contents)?;

                    println!("Remote is already a share, aborting...");

                    fs::remove_dir_all(share_dir)?;
                }
            }

            println!("Done!");
        },
        _ => (),
    };

    Ok(())
}