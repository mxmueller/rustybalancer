use std::env;
use futures_util::StreamExt;
use std::sync::Arc;
use dotenv::dotenv;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio::time::{sleep, Duration};
use tokio::sync::RwLock;
use log::{info, error, warn};

use crate::queue::{read_queue, QueueItem};

pub type SharedState = Arc<RwLock<Option<Vec<QueueItem>>>>;

pub async fn connect_socket(shared_state: SharedState) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let ws_env_port = env::var("HOST_PORT_WS_DEPLOYMENT_AGENT")
        .expect("HOST_PORT_WS_DEPLOYMENT_AGENT must be set")
        .parse::<u16>()
        .expect("HOST_PORT_WS_DEPLOYMENT_AGENT must be a valid u16");

    let url_string = format!("ws://deployment-agent:{}/ws", ws_env_port);
    let url = url_string.as_str();

    let mut retry_delay = Duration::from_secs(1);
    let max_retry_delay = Duration::from_secs(60);

    // Main loop for websocket connection and messages
    loop {
        info!("Attempting to connect to WebSocket at {}", url);
        match connect_async(url).await {
            Ok((mut ws_stream, _)) => {
                info!("Connected to the WebSocket");
                retry_delay = Duration::from_secs(1);  // Reset retry delay on successful connection

                // Incoming websocket messages
                while let Some(msg) = ws_stream.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            tokio::spawn({
                                let shared_state = shared_state.clone();
                                let text = text.clone();
                                async move {
                                    match read_queue(&text) {
                                        Ok(queue_items) => {
                                            let mut state = shared_state.write().await;
                                            *state = Some(queue_items);
                                            info!("Updated queue state");
                                        }
                                        Err(e) => {
                                            error!("Failed to parse queue data: {}", e);
                                        }
                                    }
                                }
                            });
                        }
                        Ok(_) => warn!("Received non-text message from WebSocket"),
                        Err(e) => {
                            error!("Error receiving message: {}. Reconnecting...", e);
                            break; // retrying connection
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to WebSocket: {}. Retrying in {} seconds...", e, retry_delay.as_secs());
                sleep(retry_delay).await;
                retry_delay = std::cmp::min(retry_delay * 2, max_retry_delay);
            }
        }
    }
}