use serde::{Deserialize, Serialize};
use serde_json::from_str;
use tokio_tungstenite::tungstenite::Message;
use crate::socket::WebSocketReceiver;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueItem {
    name: String,
}

pub async fn read_queue(ws_rx: WebSocketReceiver) {
    while let Some(queue_data) = ws_rx.lock().await.recv().await {
        match queue_data {

            Message::Text(text) => {
                // Handle the JSON text message
                match from_str::<Vec<QueueItem>>(&text) {
                    Ok(queue_items) => {
                        if let Some(first_item) = queue_items.first() {
                            println!("First item in the queue: {:?}", first_item);
                            // Process the first item
                        } else {
                            println!("Queue is empty");
                        }
                    }
                    Err(e) => {
                        println!("Failed to deserialize JSON: {}", e);
                    }
                }
            }
            _ => {
                println!("Couldn't received queue data from websocket.");
            }

        }
    }
}


