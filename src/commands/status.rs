use std::time::{SystemTime, UNIX_EPOCH};

use crate::provider::GpuProvider;
use crate::provider::vast::VastClient;
use crate::state::State;
use anyhow::Result;

pub async fn run(client: &VastClient) -> Result<()> {
    match State::load()? {
        None => println!("no instance managed"),
        Some(state) => match client.get_instance(&state.instance_id).await? {
            None => println!("instance {} not found on Vast.ai", state.instance_id),
            Some(instance) => {
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                let elapse_minutes = (current_time - state.created_at) as f64 / 60.0;
                let cost = elapse_minutes / 60.0 * instance.price_per_hour;
                println!(
                    "current session cost: ${:.2}({:.0}m elapsed)",
                    cost, elapse_minutes
                );
                println!(
                    "GPU Name: {}, Actual Status: {:?}, DPH Total: {}, Public IP: {:?}",
                    instance.gpu_name.as_deref().unwrap_or("unknown"),
                    instance.status,
                    instance.price_per_hour,
                    instance.public_ip,
                );
            }
        },
    }
    Ok(())
}
