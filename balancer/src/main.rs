use tokio::sync::mpsc;
use crate::http::http;
use crate::socket::{socket};
use tokio::sync::mpsc::{Receiver};

mod socket;
mod http;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    tokio::spawn(async move {
        http::http(tx).await;
    });

    tokio::spawn(async move {
        socket(tx).await;
    });

    while let Some(message) = rx.recv().await {
        println!("Received request: {}", message);
    }



}
