use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::State;
use crate::vast_client::VastClient;
use anyhow::Result;

pub async fn run(client: &VastClient) -> Result<()> {
    match State::load()? {
        None => println!("no instance managed"),
        Some(state) => match client.get_instance(state.instance_id).await? {
            None => println!("instance {} not found on Vast.ai", state.instance_id),
            Some(instance) => {
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                let elapse_minutes = (current_time - state.created_at) as f64 / 60.0;
                let cost = elapse_minutes / 60.0
                    * instance
                        .search
                        .as_ref()
                        .map(|s| s.total_hour)
                        .unwrap_or(instance.dph_total);
                println!(
                    "current session cost: ${:.2}({:.0}m elapsed)",
                    cost, elapse_minutes
                );
                println!(
                    "GPU Name: {}, Actual Status: {}, DPH Total: {}, Public IP: {}",
                    instance.gpu_name.as_deref().unwrap_or("unknown"),
                    instance.actual_status.as_deref().unwrap_or("unknow"),
                    instance
                        .search
                        .as_ref()
                        .map(|s| s.total_hour)
                        .unwrap_or(instance.dph_total),
                    instance.public_ipaddr.as_deref().unwrap_or("unknown"),
                );
            }
        },
    }
    Ok(())
}
