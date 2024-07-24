use std::env;
use futures_util::StreamExt;
use std::sync::{Arc, Mutex};
use dotenv::dotenv;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use tokio::task;
use tokio::time::{sleep, Duration};

pub type SharedState = Arc<Mutex<String>>;

pub async fn connect_socket(shared_state: SharedState) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let ws_env_port = env::var("HOST_PORT_WS_DEPLOYMENT_AGENT")
        .expect("HOST_PORT_WS_DEPLOYMENT_AGENT must be set")
        .parse::<u16>()
        .expect("HOST_PORT_WS_DEPLOYMENT_AGENT must be a valid u16");

    let url_string = format!("ws://deployment-agent:{}/ws", ws_env_port);
    let url = Url::parse(&url_string)?;

    loop {
        match connect_async(&url).await {
            Ok((mut ws_stream, _)) => {
                println!("Connected to the Socket");

                let (_, mut read) = ws_stream.split();
                let state = shared_state.clone();

                // Task for reading messages
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            let mut state = state.lock().unwrap();
                            *state = text;
                        }
                        Ok(_) => (),
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
