use std::sync::Arc;
use tokio::sync::RwLock;

mod socket;
mod http;
mod queue;
mod pool;

use crate::http::start_http_server;
use crate::socket::connect_socket;
use crate::pool::ConnectionPool;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let shared_state = Arc::new(RwLock::new(None));
    let connection_pool = ConnectionPool::new(100); // Anpassen der Poolgröße nach Bedarf

    let ws_state = shared_state.clone();
    tokio::spawn(async move {
        if let Err(e) = connect_socket(ws_state).await {
            eprintln!("WebSocket connection error: {}", e);
        }
    });

    let http_state = shared_state.clone();
    let http_pool = connection_pool.clone();
    tokio::spawn(async move {
        if let Err(e) = start_http_server(http_state, http_pool).await {
            eprintln!("HTTP server error: {}", e);
        }
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}