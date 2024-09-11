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
use crate::queue::{build_queue};

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

    loop {
        // Gets current queue
        match build_queue().await {
            Ok(shared_queue) => {
                let locked_queue = shared_queue.lock().await;
                let queue_string = serde_json::to_string(&*locked_queue).expect("Failed to serialize queue");

                // Sends serialized queue with the websocket
                if let Err(e) = socket.send(Message::Text(queue_string.clone())).await {
                    eprintln!("Error sending message: {}", e);
                    // Breaks loop, if an error occurred
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to build queue: {:?}", e);
                break;
            }
        }

        // Waits 2 secs before sending update
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    println!("WebSocket connection closed");
}