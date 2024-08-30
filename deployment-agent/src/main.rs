use bollard::errors::Error;
use dotenv::dotenv;
use http::start_http_server;
use crate::socket::socket;

mod container;
mod stats;
mod http;
mod queue;
mod socket;
mod db;

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let mut conn = db::get_redis_connection();
    db::init(&mut conn);

    println!("Starting HTTP server...");
    let http_server = tokio::spawn(async {
        start_http_server().await;
    });

    println!("Starting socket...");
    let socket_task = tokio::spawn(async {
        socket().await;
    });

    // Await both tasks
    let (http_result, socket_result) = tokio::join!(http_server, socket_task);

    if let Err(e) = http_result {
        eprintln!("HTTP server task failed: {:?}", e);
    }

    if let Err(e) = socket_result {
        eprintln!("Socket task failed: {:?}", e);
    }

    Ok(())
}