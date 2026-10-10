mod cli;
mod manifest;
mod git;
mod share;
mod listener;

use crate::cli::*;
use crate::share::*;
use crate::listener::*;

use clap::Parser;
use std::env;
use anyhow::{Context, Result};
use directories::ProjectDirs;

fn main() -> Result<()> {
    let args = Args::parse();

    let dirs = ProjectDirs::from("com", "TheGirl790", "dirshare")
        .expect("Could not determine config directory");

    let mut config_path = dirs.config_dir().to_path_buf();
    config_path.push("listener.toml");

    let config = if config_path.exists() {
        ListenerConfig::load(&config_path)?
    } else {
        println!("Creating new global config.");
        ListenerConfig::default()
    };

    config.write(&config_path)?;

    let working_dir = &env::current_dir().context("Failed to get working directory.")?;

    match args.command {
        Command::New { share_name, remote } => {
            new_share(
                working_dir,
                &config_path,
                share_name,
                remote,
            )
        },
        Command::Join { remote } => {
            join_share(
                working_dir,
                &config_path,
                remote,
            )
        },
        Command::Listen => {
            let mut listener = ShareListener::new(&config_path)?;
            listener.start(config_path)?;

            Ok(())
        }
        _ => Ok(()),
    }?;

    Ok(())
}