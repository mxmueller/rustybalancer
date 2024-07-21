use bollard::errors::Error;
use dotenv::dotenv;
use std::sync::Arc;
use tokio::sync::Mutex;
use stats::get_container_status;
use http::{start_http_server, ContainerStatus, AppState};
use crate::queue::build_queue;
use crate::socket::socket;

mod container;
mod stats;
mod http;
mod queue;
mod socket;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    // Starten Sie Ihre Container (Optional)
    container::start_containers().await?;

    // Container-Status abrufen
    let status = get_container_status().await?;
    let container_stats = status.into_iter().map(|stat| ContainerStatus {
        id: stat.id,
        name: stat.name,
        image: stat.image,
        state: stat.state,
        ports: stat.ports,
        cpu_usage: stat.cpu_usage,
        memory_usage: stat.memory_usage, 
        uptime: stat.uptime,
    }).collect::<Vec<_>>();

    let app_state = AppState {
        container_stats: Arc::new(Mutex::new(container_stats)),
    };

    let mut queue = build_queue().await;


    // Start the HTTP server
    start_http_server(app_state).await;

    // Send messages to the websocket
    socket(queue).await;


    Ok(())
}
