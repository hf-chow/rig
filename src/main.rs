use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::config::Config;
use crate::vast_client::VastClient;

mod commands;
mod config;
mod provider;
mod state;
mod vast_client;

#[derive(Subcommand, Clone)]
enum Command {
    Status,
    Up,
    Down,
    Ssh,
}

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;
    let token = config.token()?;
    let client = VastClient::new(token);
    match cli.command {
        Command::Status => commands::status::run(&client).await?,
        Command::Up => commands::up::run(&client, &config).await?,
        Command::Down => commands::down::run(&client).await?,
        Command::Ssh => commands::ssh::run().await?,
    };
    Ok(())
}
