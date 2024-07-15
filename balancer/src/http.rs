use std::net::SocketAddr;
use std::time::Instant;

use axum::{
    http::Request,
    middleware::{self, Next},
    response::{Html, IntoResponse},
    Router,
    routing::get,
};
use tokio::sync::mpsc::Sender;
use tower::ServiceBuilder;

async fn greet() -> impl IntoResponse {
    let html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Landing Page</title>
        </head>
        <body>
            <h1>Hello, this is the landing page!</h1>
        </body>
        </html>
    "#;

    Html(html)
}

pub async fn http(tx: Sender<String>) {
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
        .route("/", get(greet))
        .layer(middleware_stack);

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
