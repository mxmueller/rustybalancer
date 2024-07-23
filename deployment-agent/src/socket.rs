use std::env;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::time::Duration;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use crate::queue::SharedQueue;

pub async fn socket(queue: SharedQueue) -> (mpsc::Sender<String>, mpsc::Receiver<String>) {
    let (tx, rx) = mpsc::channel(32);

    dotenv().ok();
    let ws_env_port = env::var("HOST_PORT_WS_DEPLOYMENT_AGENT")
        .expect("HOST_PORT_WS_DEPLOYMENT_AGENT must be set")
        .parse::<u16>()
        .expect("HOST_PORT_WS_DEPLOYMENT_AGENT must be a valid u16");

    let addr_ws = SocketAddr::from(([0, 0, 0, 0], ws_env_port));
    // Router with get (for HTTP GET).
    // http_to_ws without round-brackets, because it's referring to the pointer of the function.
    let app = Router::new().route("/ws", get(move |ws: WebSocketUpgrade| http_to_ws(ws, queue.clone())));
    let addr = SocketAddr::from(([0, 0, 0, 0], 2548));

    println!("Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    (tx, rx)
}

async fn http_to_ws(ws: WebSocketUpgrade, queue: SharedQueue) -> impl IntoResponse {
    println!("WebSocket upgrade requested");
    ws.on_upgrade(move |socket| handle_socket(socket, queue))
}

#[derive(Serialize, Deserialize)]
pub enum Event {
    Echo { message: String },
    // Other variants...
}

async fn handle_socket(mut socket: WebSocket, queue: SharedQueue) {
    println!("WebSocket connection established");
    // Await to wait for the next element of the stream to be available.
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    // Task for sending the entire queue at regular intervals
    let queue_sender = queue.clone();
    tokio::spawn(async move {
        loop {
            // Lock the queue and serialize it
            let queue_data = {
                let queue = queue_sender.lock().await;
                serde_json::to_string(&*queue).expect("Failed to serialize queue")
            };

            // Send the serialized queue as a single message
            if let Err(e) = socket.send(Message::Text(queue_data)).await {
                eprintln!("Error sending message: {}", e);
                return;
            }

            // Wait before sending the queue again
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });

    /*
    loop {
        // Periodically send messages
        interval.tick().await; // waiting for the next tick
            let response = "Periodic update message".to_string();
            if let Err(e) = socket.send(Message::Text(response)).await {
                eprintln!("Error sending periodic message: {}", e);
                return;
            }
    }
     */
}
