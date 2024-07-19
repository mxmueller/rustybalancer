use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::time::Duration;
use serde::{Deserialize, Serialize};

pub async fn socket() {
    // Router with get (for HTTP GET).
    // http_to_ws without round-brackets, because it's referring to the pointer of the function.
    let app = Router::new().route("/ws", get(http_to_ws));
    let addr = SocketAddr::from(([0, 0, 0, 0], 2547));

    println!("Server listening on {}", addr);

    axum::Server::bind(&addr)
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
    // Await to wait for the next element of the stream to be available.
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        // Periodically send messages
        interval.tick().await; // waiting for the next tick
            let response = "Periodic update message".to_string();
            if let Err(e) = socket.send(Message::Text(response)).await {
                eprintln!("Error sending periodic message: {}", e);
                return;
            }
    }
}
