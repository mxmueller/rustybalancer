use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use crate::stats;
use std::net::SocketAddr;

pub async fn serve() {
    // Build our application with a single route
    let app = Router::new().route("/stats", get(get_stats));

    // Run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_stats() -> impl IntoResponse {
    match stats::display_stats().await {
        Ok(stats) => (
            StatusCode::OK,
            Json(json!({ "status": "success", "data": stats })),  // Serialize stats to JSON
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": e.to_string() })),
        ),
    }
}
