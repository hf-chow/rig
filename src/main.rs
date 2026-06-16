use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use crate::{
    config::{Config, RunPodConfig},
    provider::{GpuProvider, runpod::RunPodClient, vast::VastClient},
};

mod commands;
mod config;
mod provider;
mod state;

#[derive(Subcommand, Clone)]
enum Command {
    Status,
    Up,
    Down,
    Ssh,
}

#[derive(Clone, ValueEnum)]
enum Provider {
    Vastai,
    Runpod,
}

#[derive(Parser)]
pub struct Cli {
    #[arg(long, value_enum, default_value_t = Provider::Vastai)]
    pub provider: Provider,
    #[command(subcommand)]
    pub command: Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;
    let vast_cfg = config.providers.vastai.clone().unwrap_or_default();
    let client: Box<dyn GpuProvider> = match cli.provider {
        Provider::Vastai => {
            let cfg = config.providers.vastai.clone().unwrap_or_default();
            Box::new(VastClient::new(
                cfg.resolve_token()?,
                cfg.ssh_key_id.unwrap_or_default(),
            ))
        }
        Provider::Runpod => {
            let cfg = config.providers.runpod.clone().unwrap_or_default();
            Box::new(RunPodClient::new(cfg.resolve_token()?))
        }
    };
    match cli.command {
        Command::Status => commands::status::run(&*client).await?,
        Command::Up => commands::up::run(&*client, &config).await?,
        Command::Down => commands::down::run(&*client).await?,
        Command::Ssh => commands::ssh::run().await?,
    };
    Ok(())
}
