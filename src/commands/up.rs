use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::state::State;
use crate::vast_client::VastClient;
use anyhow::Result;

pub async fn run(client: &VastClient, config: &Config) -> Result<()> {
    if let Some(state) = State::load()? {
        match client.get_instance(state.instance_id).await? {
            Some(instance) if instance.actual_status.as_deref() == Some("running") => {
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                let elapse_minutes = (current_time - state.created_at) as f64 / 60.0;
                let cost = elapse_minutes / 60.0
                    * instance
                        .search
                        .as_ref()
                        .map(|s| s.total_hour)
                        .unwrap_or(instance.dph_total);
                println!(
                    "instance {} has been up for {:.0}m:  session cost so far: ${:.2}",
                    instance.id, elapse_minutes, cost
                );
            }
            _ => {
                println!("stale state found, clearing...");
                State::clear()?
            }
        }
        return Ok(());
    }
    let offers = client
        .list_offers(
            config.min_vram_gb.unwrap_or(24),
            config.max_price_per_hour.unwrap_or(0.5),
        )
        .await?;
    if offers.is_empty() {
        anyhow::bail!("no matching offers found")
    }
    let offer = offers.into_iter().next().unwrap();
    println!(
        "renting {} with {} VRAM at  ${}/hr",
        offer.gpu_name.as_deref().unwrap_or("unknown"),
        offer.gpu_ram,
        offer.search.total_hour,
    );
    let instance_id = client
        .create_instance(
            offer.id,
            config
                .image
                .as_deref()
                .unwrap_or("pytorch/pytorch:2.3.0-cuda12.1-cudnn8-runtime"),
            offer.disk_space,
            config.ssh_key_id.as_deref().unwrap_or(&[]),
        )
        .await?;
    let mut state = State {
        instance_id,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        ip: None,
        ssh_host: None,
        ssh_port: None,
    };
    state.save()?;

    println!("waiting for instance to start...");
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        match client.get_instance(instance_id).await? {
            None => anyhow::bail!("somehing is wrong when starting an instance"),
            Some(instance) => {
                if instance.actual_status.as_deref() == Some("running") {
                    state.ip = instance.public_ipaddr;
                    state.ssh_port = instance.ssh_port;
                    state.ssh_host = instance.ssh_host;
                    state.save()?;
                    println!("instance ready at {}", state.ip.unwrap());
                    break;
                }
                println!(
                    "status: {}, waiting...",
                    instance.actual_status.as_deref().unwrap_or("unknown")
                );
            }
        };
    }
    println!("instance created, id: {}", instance_id);
    Ok(())
}
