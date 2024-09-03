use serde::{Deserialize, Serialize};
use serde_json::from_str;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct QueueItem {
    pub dns_name: String,
    pub score: f64,
    pub utilization_category: String,
}

pub fn read_queue(text: &str) -> Result<Vec<QueueItem>, String> {
    match from_str::<Vec<QueueItem>>(text) {
        Ok(queue_items) => Ok(queue_items),
        Err(e) => Err(format!("Failed to deserialize JSON: {}", e)),
    }
}