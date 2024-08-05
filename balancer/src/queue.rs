use serde::{Deserialize, Serialize};
use serde_json::from_str;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueItem {
    pub name: String,
    pub external_port: String,
}

pub fn read_queue(text: &str) -> Result<Vec<QueueItem>, String> {
    match from_str::<Vec<QueueItem>>(text) {
        Ok(queue_items) => Ok(queue_items),
        Err(e) => Err(format!("Failed to deserialize JSON: {}", e)),
    }
}
