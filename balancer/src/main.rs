use crate::http::start_http_server;
use crate::socket::connect_socket;
use std::sync::Arc;
use tokio::sync::Mutex;

mod socket;
mod http;
mod queue;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let shared_state = Arc::new(Mutex::new(None));

    let ws_state = shared_state.clone();
    tokio::spawn(async move {
        if let Err(e) = connect_socket(ws_state).await {
            eprintln!("WebSocket connection error: {}", e);
        }
    });

    let http_state = shared_state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_http_server(http_state).await {
            eprintln!("HTTP server error: {}", e);
        }
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
