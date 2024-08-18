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
use tokio::sync::Mutex;
use crate::queue::{SharedQueue, build_queue};

pub async fn socket() {
    dotenv().ok();
    let ws_env_port = env::var("HOST_PORT_WS_DEPLOYMENT_AGENT")
        .expect("HOST_PORT_WS_DEPLOYMENT_AGENT must be set")
        .parse::<u16>()
        .expect("HOST_PORT_WS_DEPLOYMENT_AGENT must be a valid u16");

    let addr_ws = SocketAddr::from(([0, 0, 0, 0], ws_env_port));
    let app = Router::new().route("/ws", get(http_to_ws));

    println!("Socket listening on {}", addr_ws);

    axum::Server::bind(&addr_ws)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn http_to_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    println!("WebSocket upgrade requested");
    ws.on_upgrade(handle_socket)
}

#[derive(Serialize, Deserialize)]
pub enum Event {
    Echo { message: String },
    // Other variants...
}

async fn handle_socket(mut socket: WebSocket) {
    println!("WebSocket connection established");

    tokio::spawn(async move {
        loop {
            // Hole die aktuelle Queue
            match build_queue().await {
                Ok(shared_queue) => {
                    let locked_queue = shared_queue.lock().await;
                    let queue_string = serde_json::to_string(&*locked_queue).expect("Failed to serialize queue");

                    // Sende die serialisierte Queue
                    if let Err(e) = socket.send(Message::Text(queue_string)).await {
                        eprintln!("Error sending message: {}", e);
                        // Breche die Schleife ab, falls ein Fehler auftritt
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to build queue: {:?}", e);
                    break;
                }
            }

            // Warte vor dem Senden des nächsten Updates
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    /*
    // Task for receiving messages from the WebSocket
    tokio::spawn(async move {
        while let Some(Ok(message)) = socket.recv().await {
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
