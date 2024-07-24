use crate::http::http;
use crate::socket::{connect_socket, SharedState};
use std::sync::{Arc, Mutex};

mod socket;
mod http;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let shared_state = Arc::new(Mutex::new(String::new()));

    let ws_state = shared_state.clone();
    tokio::spawn(async move {
        if let Err(e) = connect_socket(ws_state).await {
            eprintln!("WebSocket connection error: {}", e);
        }
    });

    let http_state = shared_state.clone();
    tokio::spawn(async move {
        if let Err(e) = http(http_state).await {
            eprintln!("HTTP server error: {}", e);
        }
    });

    loop { // refresh lifetime all 60 secs
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
