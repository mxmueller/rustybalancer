use std::env;
use std::sync::Arc;
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
use tokio::sync::{mpsc, Mutex};
use tokio::time::timeout;
use crate::queue::SharedQueue;

pub async fn socket(queue: SharedQueue) -> (mpsc::Sender<String>, mpsc::Receiver<String>) {
    let (tx, rx) = mpsc::channel(32);

    dotenv().ok();
    let ws_env_port = env::var("HOST_PORT_WS_DEPLOYMENT_AGENT")
        .expect("HOST_PORT_WS_DEPLOYMENT_AGENT must be set")
        .parse::<u16>()
        .expect("HOST_PORT_WS_DEPLOYMENT_AGENT must be a valid u16");

    let addr_ws = SocketAddr::from(([0, 0, 0, 0], ws_env_port));
    let app = Router::new().route("/ws", get(move |ws: WebSocketUpgrade| http_to_ws(ws, queue.clone())));

    println!("Socket listening on {}", addr_ws);

    axum::Server::bind(&addr_ws)
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

async fn handle_socket(socket: WebSocket, queue: SharedQueue) {
    println!("WebSocket connection established");
    let socket = Arc::new(Mutex::new(socket));

    // Task for sending the entire queue at regular intervals
    let queue_sender = queue.clone();
    let socket_sender = Arc::clone(&socket);
    tokio::spawn(async move {
        loop {
            // Lock the queue and serialize it
            let queue_data = {
                let queue = queue_sender.lock().await;
                let queue_string = serde_json::to_string(&*queue).expect("Failed to serialize queue");
                // println!("QUEUE: {}", queue_string);
                queue_string
            };
            println!("{}", queue_data);
            // Send the serialized queue as a single message
            // Try to lock the socket with a timeout
            match timeout(Duration::from_secs(5), socket_sender.lock()).await {
                Ok(mut socket_lock) => {
                    if let Err(e) = socket_lock.send(Message::Text(queue_data.clone())).await {
                        eprintln!("Error sending message: {}", e);
                        // Break the loop if an error occurs to avoid further broken pipe errors
                        break;
                    }
                    // println!("Successfully sent queue with websocket: {}.", queue_data);
                }
                Err(_) => {
                    eprintln!("Timeout while trying to lock the socket");
                }
            }
            // Wait before sending the queue again
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    });
    /*
    // Task for receiving messages from the WebSocket
    let socket_receiver = Arc::clone(&socket);
    tokio::spawn(async move {
        while let Some(Ok(message)) = socket_receiver.lock().await.recv().await {
            match message {
                Message::Text(text) => {
                    println!("Received message: {}", text);
                    // Handle received message
                }
                Message::Close(_) => {
                    println!("WebSocket connection closed");
                    break;
                }
                _ => {}
            }
        }
    });

     */
}
