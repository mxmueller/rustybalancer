use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::stream::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;


pub async fn socket() {
    // Initialize the router with the WebSocket route.
    let app = Router::new().route("/ws", get(handle_ws_upgrade));

    // Define the address for the server to listen on.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Start the server.
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Handler to upgrade HTTP connection to WebSocket connection.
async fn handle_ws_upgrade(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

// Handle the WebSocket connection.
async fn handle_socket(socket: WebSocket) {
    let socket = Arc::new(Mutex::new(socket));

    while let Some(msg) = socket.lock().await.next().await {
        match msg {
            Ok(msg) => {
                let socket = Arc::clone(&socket);
                task::spawn(async move {
                    handle_message(msg, socket).await;
                });
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                return;
            }
        }
    }
}

// Function to handle individual messages.
async fn handle_message(msg: Message, socket: Arc<Mutex<WebSocket>>) {
    match msg {
        Message::Text(text) => {
            // Echo the received text message back to the client.
            if let Err(e) = socket.lock().await.send(Message::Text(text)).await {
                eprintln!("Error sending message: {}", e);
            }
        }
        _ => {}
    }
}
