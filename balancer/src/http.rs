use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use axum::response::{Html, IntoResponse};

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

pub async fn http() {
    // Creates Router.
    let app = Router::new()
        .route("/", get(greet));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8090));
    println!("Listening on {}", addr);

    // Starte den Server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

