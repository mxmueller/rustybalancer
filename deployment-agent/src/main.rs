mod socket;
mod swarm;
mod stats;
mod http;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    if let Err(e) = swarm::swarm_boot().await {
        eprintln!("Error: {}", e);
    }

    // Start the HTTP server
    http::serve().await;

    // socket::socket().await;
}
