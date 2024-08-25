use std::env;
use futures_util::StreamExt;
use std::sync::Arc;
use dotenv::dotenv;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio::time::{sleep, Duration};
use tokio::sync::RwLock;

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

    loop {
        match connect_async(url).await {
            Ok((mut ws_stream, _)) => {
                println!("Connected to the Socket");

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
                                        }
                                        Err(e) => {
                                            println!("{}", e);
                                        }
                                    }
                                }
                            });
                        }
                        Ok(_) => println!("Unknown data received."),
                        Err(e) => {
                            eprintln!("Error receiving message: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to Socket: {}. Retrying in 5 seconds...", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}