use axum::{
    extract::Extension,
    http::Request,
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::time::Instant;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::Message;
use tower::{ServiceBuilder};
use crate::socket::{WebSocketReceiver, WebSocketSender};

async fn get_ws_queue(
    Extension(ws_rx): Extension<WebSocketReceiver>,
    Extension(ws_tx): Extension<WebSocketSender>
) -> impl IntoResponse {
    // Reading the latest message from the Websocket
    let mut ws_rx = ws_rx.lock().await;

    let latest_message = if let Some(msg) = ws_rx.recv().await {
        match msg {
            Message::Text(text) => text,
            _ => "Received non-text message".to_string(),
        }
    } else {
        "No messages".to_string()
    };

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Landing Page</title>
        </head>
        <body>
            <h1>Hello, this is the landing page!</h1>
            <p>Latest message from WebSocket: {}</p>
        </body>
        </html>
    "#,
        latest_message
    );

    /*
    // Sending an event to the WebSocket
    let message = "HTTP request received on /".to_string();
    let ws_tx = ws_tx.clone();
    tokio::spawn(async move {
        let mut ws_tx = ws_tx.lock().await;
        if let Err(e) = ws_tx.send(message).await {
            eprintln!("Error sending WebSocket message: {}", e);
        }
    });
    */

    Html(html)
}

pub async fn http(
    tx: Sender<String>,
    ws_sender: WebSocketSender,
    ws_receiver: WebSocketReceiver
) {
    // Creates middleware stack  with a clone of the sender (tx) to log requests.
    let middleware_stack = ServiceBuilder::new()
        .layer(middleware::from_fn(move |req, next| {
            let tx = tx.clone();
            async move {
                log_request(req, next, tx).await
            }
        }));
    // Creates Router.
    let app = Router::new()
        .route("/", get(get_ws_queue))
        .layer(middleware_stack)
        .layer(Extension(ws_receiver))
        .layer(Extension(ws_sender));

    let addr = SocketAddr::from(([0, 0, 0, 0], 2548));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Logs each incoming request with the time needed to handle the request.
async fn log_request<B>(req: Request<B>, next: Next<B>, tx: Sender<String>) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();

    // Sends message to the sender.
    if let Err(e) = tx.send(format!("{} {}", method, uri)).await {
        eprintln!("Failed to send message: {}", e);
    }

    let response = next.run(req).await;

    let duration = start.elapsed();
    println!("{} {} - {:?}", method, uri, duration);

    response
}
