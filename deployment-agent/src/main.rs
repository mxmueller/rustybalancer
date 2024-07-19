mod socket;
mod swarm;
mod stats;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    if let Err(e) = swarm::swarm_boot().await {
        eprintln!("Error: {}", e);
    }

    println!("FOOOO");

    if let Err(e) = stats::display_stats().await {
        eprintln!("Error: {}", e);
    } else {
        println!("FOOOO");
    }

    //socket::socket().await;
}
