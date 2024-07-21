use axum::{routing::get, Router, extract::State, response::Json};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::net::SocketAddr;

#[derive(Serialize, Clone)]
pub struct ContainerStatus {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub ports: std::collections::HashMap<String, Vec<String>>,
    pub cpu_usage: f64,
    pub memory_usage: f64, // Hier geändert zu f64
    pub uptime: String,
}

#[derive(Clone)]
pub struct AppState {
    pub container_stats: Arc<Mutex<Vec<ContainerStatus>>>,
}

pub async fn start_http_server(state: AppState) {
    let app = Router::new()
        .route("/stats", get(get_stats))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_stats(State(state): State<AppState>) -> Json<Vec<ContainerStatus>> {
    let stats = state.container_stats.lock().await;
    Json(stats.clone())
}
