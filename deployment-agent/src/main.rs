use bollard::errors::Error;
use dotenv::dotenv;
use http::start_http_server;
use crate::queue::build_queue;
use crate::socket::socket;
use tokio::task::JoinError;

mod container;
mod stats;
mod http;
mod queue;
mod socket;

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    println!("Starting HTTP server...");
    let http_server = tokio::spawn(async {
        start_http_server().await;
    });

    println!("Building queue...");
    let queue = build_queue().await;
    println!("Queue built.");

    println!("Starting socket...");
    let socket_task = tokio::spawn(async {
        socket(queue).await;
    });

    println!("Starting containers...");
    let containers_task = tokio::spawn(async {
        container::start_containers().await?;
        Ok::<(), Error>(())
    });

    // Await the HTTP server task and handle it separately
    let http_result = http_server.await;
    if let Err(e) = http_result {
        eprintln!("HTTP server task failed: {:?}", e);
    }

    // Await the other tasks
    match tokio::try_join!(socket_task, containers_task) {
        Ok(_) => println!("Socket and containers tasks completed."),
        Err(e) => eprintln!("An error occurred: {:?}", e),
    }

    Ok(())
}
