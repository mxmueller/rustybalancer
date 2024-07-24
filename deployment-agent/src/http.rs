use axum::{
    response::Json,
    routing::get,
    Router,
};
use std::env;
use std::net::SocketAddr;
use dotenv::dotenv;
use tower_http::cors::{Any, CorsLayer};
use crate::stats::{get_container_status, ContainerStatus};  // Importiere ContainerStatus aus stats.rs

pub async fn start_http_server() {
    let app = Router::new()
        .route("/stats", get(get_stats))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    dotenv().ok();
    let http_env_port = env::var("HOST_PORT_HTTP_DEPLOYMENT_AGENT")
        .expect("HOST_PORT_HTTP_DEPLOYMENT_AGENT must be set")
        .parse::<u16>()
        .expect("HOST_PORT_HTTP_DEPLOYMENT_AGENT must be a valid u16");

    let addr = SocketAddr::from(([0, 0, 0, 0], http_env_port));
    println!("HTTP Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_stats() -> Result<Json<Vec<ContainerStatus>>, axum::http::StatusCode> {
    match get_container_status().await {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}
