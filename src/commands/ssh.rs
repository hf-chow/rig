use crate::state::State;
use anyhow::Result;
use std::process::Command;

pub async fn run() -> Result<()> {
    let state = match State::load()? {
        None => {
            println!("no instance managed");
            return Ok(());
        }
        Some(state) => state,
    };
    let status = Command::new("ssh")
        .arg("-p")
        .arg(state.ssh_port.unwrap_or(22).to_string())
        .arg(format!(
            "root@{}",
            state.ssh_host.as_deref().unwrap_or("unknown")
        ))
        .env("TERM", "xterm-256color")
        .status()?;
    if !status.success() {
        anyhow::bail!("ssh existed with non-zero status");
    }
    Ok(())
}
