use std::sync::Arc;
use tokio::sync::mpsc;
use crate::http::http;
use crate::queue::read_queue;
use crate::socket::{connect_socket};

mod socket;
mod http;
mod queue;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let (ws_tx, ws_rx) = connect_socket().await;

    let (log_tx, mut log_rx) = mpsc::channel(32);

    let shared_ws_rx = Arc::clone(&ws_rx);
    let shared_ws_tx = Arc::clone(&ws_tx);

    tokio::spawn(async move {
        http(log_tx, shared_ws_tx, shared_ws_rx).await;
    });

    // Task for processing incoming WebSocket messages
    tokio::spawn({
        let ws_rx = ws_rx.clone();
        async move {
            read_queue(ws_rx).await;
        }
    });

    tokio::spawn(async move {
        while let Some(log) = log_rx.recv().await {
            println!("Log: {}", log);
        }
    });



}
