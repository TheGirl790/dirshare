use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Command {
    New { share_name: String, remote: String },
    Join { remote: String },
    Status,
    Start,
    Stop,
}

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command
}
