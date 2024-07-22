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

    let mut queue = build_queue().await;

    start_http_server().await;

    // Send messages to the websocket
    socket(queue).await;

    Ok(())
}
