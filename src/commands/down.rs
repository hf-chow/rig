use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::State;
use crate::vast_client::VastClient;
use anyhow::Result;

pub async fn run(client: &VastClient) -> Result<()> {
    let state = match State::load()? {
        None => {
            println!("no instance managed");
            return Ok(());
        }
        Some(s) => s,
    };
    let _instance = match client.get_instance(state.instance_id).await? {
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
                "session cost: ${:.2} ({:.0}m elapsed)",
                cost, elapse_minutes
            );
            let destory_result = client.destroy_instance(state.instance_id).await;
            State::clear()?;
            destory_result?;
        }
        None => println!("instance already destroyed, clearing state"),
    };
    State::clear()?;
    println!("instance {} dismantling complete", state.instance_id);
    Ok(())
}
