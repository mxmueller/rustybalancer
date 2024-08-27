use std::sync::Arc;
use tokio::sync::RwLock;

mod socket;
mod http;
mod queue;
mod pool;
mod cache;  // Add this line to import the new cache module

use crate::http::start_http_server;
use crate::socket::connect_socket;
use crate::pool::ConnectionPool;
use crate::cache::SimpleCache;  // Add this line to import SimpleCache

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    let shared_state = Arc::new(RwLock::new(None));
    let connection_pool = ConnectionPool::new(5000);
    let cache = Arc::new(SimpleCache::new(10000));

    let ws_state = shared_state.clone();
    tokio::spawn(async move {
        if let Err(e) = connect_socket(ws_state).await {
            eprintln!("WebSocket connection error: {}", e);
        }
    });

    let http_state = shared_state.clone();
    let http_pool = connection_pool.clone();
    let http_cache = cache.clone();
    if let Err(e) = start_http_server(http_state, http_pool, http_cache).await {
        eprintln!("HTTP server error: {}", e);
    }
}