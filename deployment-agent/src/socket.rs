use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::stream::StreamExt;
use std::net::SocketAddr;

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

async fn handle_socket(mut socket: WebSocket) {
    println!("WebSocket connection established");
    // Await to wait for the next element of the stream to be available.
    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Handling text message: {}", text);
                let response = format!("Echo: {}", text);
                if let Err(e) = socket.send(Message::Text(response)).await {
                    eprintln!("Error sending message: {}", e);
                    return;
                }
            }
            Ok(_) => {
                println!("Received non-text message");
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                return;
            }
        }
    }
}
