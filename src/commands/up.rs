use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::provider::{GpuProvider, InstanceSpec, InstanceStatus, SearchCriteria};
use crate::state::State;
use anyhow::Result;

pub async fn run(client: &dyn GpuProvider, config: &Config) -> Result<()> {
    if let Some(state) = State::load()? {
        match client.get_instance(&state.instance_id).await? {
            Some(instance) if matches!(instance.status, InstanceStatus::Running) => {
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                let elapse_minutes = (current_time - state.created_at) as f64 / 60.0;
                let cost = elapse_minutes / 60.0 * instance.price_per_hour;
                println!(
                    "instance {} has been up for {:.0}m:  session cost so far: ${:.2}",
                    instance.id, elapse_minutes, cost
                );
                return Ok(());
            }
            _ => {
                println!("stale state found, clearing...");
                State::clear()?
            }
        }
    }

    let criteria = SearchCriteria {
        max_price_per_hour: config.max_price_per_hour.unwrap_or(0.1),
        min_vram_gb: config.min_vram_gb.unwrap_or(8),
        num_gpus: 1,
    };
    let offers = client.list_offers(&criteria).await?;
    if offers.is_empty() {
        anyhow::bail!("no matching offers found")
    }
    let offer = offers.into_iter().next().unwrap();
    println!(
        "renting {} with {} VRAM at  ${}/hr",
        offer.gpu_name, offer.vram_gb, offer.price_per_hour,
    );
    let spec = InstanceSpec {
        image: config.image.clone().unwrap_or("unknown".to_string()),
        disk_gb: config.disk_gb.unwrap(),
    };
    let instance_id = client.create_instance(&offer, &spec).await?;

    let mut state = State {
        instance_id: instance_id.clone(),
        created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        ip: None,
        ssh_host: None,
        ssh_port: None,
    };
    state.save()?;

    println!("waiting for instance to start...");

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        match client.get_instance(&instance_id).await? {
            None => anyhow::bail!("somehing is wrong when starting an instance"),
            Some(instance) => match instance.status {
                InstanceStatus::Running => {
                    state.ip = instance.public_ip;
                    state.ssh_port = instance.ssh_port;
                    state.ssh_host = instance.ssh_host;
                    state.save()?;
                    println!("instance ready at {}", state.ip.unwrap());
                    break;
                }
                _ => {
                    println!("status: {:?}, waiting...", instance.status)
                }
            },
        };
    }
    println!("instance created, id: {}", instance_id);
    Ok(())
}
