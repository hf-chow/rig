use std::time::{SystemTime, UNIX_EPOCH};

use crate::provider::GpuProvider;

use crate::state::State;
use anyhow::Result;

pub async fn run(client: &dyn GpuProvider) -> Result<()> {
    let state = match State::load()? {
        None => {
            println!("no instance managed");
            return Ok(());
        }
        Some(s) => s,
    };
    match client.get_instance(&state.instance_id).await? {
        Some(instance) => {
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let elapse_minutes = (current_time - state.created_at) as f64 / 60.0;
            let cost = elapse_minutes / 60.0 * instance.price_per_hour;
            println!(
                "session cost: ${:.2} ({:.0}m elapsed)",
                cost, elapse_minutes
            );
            let destory_result = client.destroy_instance(&state.instance_id).await;
            State::clear()?;
            destory_result?;
        }
        None => println!("instance already destroyed, clearing state"),
    };
    State::clear()?;
    println!("instance {} dismantling complete", state.instance_id);
    Ok(())
}
