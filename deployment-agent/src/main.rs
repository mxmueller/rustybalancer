mod socket;
mod swarm;
mod stats;

use bollard::Docker;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    if let Err(e) = swarm::swarm_boot().await {
        eprintln!("Error: {}", e);
    } else {
        let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

        match stats::get_all_container_stats(&docker).await {
            Ok(stats_map) => {
                for (id, stats) in stats_map {
                    println!("Container ID: {}", id);
                    println!("{:#?}", stats);
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    //socket::socket().await;
}
