mod cli;
mod manifest;
mod git;
mod share;

use crate::cli::*;
use crate::share::*;

use clap::Parser;
use std::env;
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let args = Args::parse();

    let working_dir = &env::current_dir().context("Failed to get working directory.")?;

    match args.command {
        Command::New { share_name, remote } => {
            new_share(
                working_dir,
                share_name,
                remote,
            )
        },
        Command::Join { remote } => {
            join_share(
                working_dir,
                remote,
            )
        }
        _ => Ok(()),
    }?;

    Ok(())
}