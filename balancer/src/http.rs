use std::env;
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use dotenv::dotenv;
use crate::socket::SharedState;

async fn landing_page() -> Html<&'static str> {
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

async fn get_ws_content(Extension(shared_state): Extension<SharedState>) -> impl IntoResponse {
    let state = shared_state.lock().unwrap();
    Html(format!("<p>Latest message from WebSocket: {}</p>", *state))
}

pub async fn http(shared_state: SharedState) -> Result<(), Box<dyn std::error::Error>> {
    // Creates Router.
    let app = Router::new()
        .route("/", get(landing_page))
        .route("/ws_content", get(get_ws_content))
        .layer(Extension(shared_state));

    dotenv().ok();
    let http_env_port = env::var("HOST_PORT_HTTP_BALANCER")
        .expect("HOST_PORT_HTTP_BALANCER must be set")
        .parse::<u16>()
        .expect("HOST_PORT_HTTP_BALANCER must be a valid u16");

    let addr = SocketAddr::from(([0, 0, 0, 0], http_env_port));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
