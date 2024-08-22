use bollard::errors::Error;
use dotenv::dotenv;
use http::start_http_server;
use crate::socket::socket;
use axum::http::StatusCode;

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

    // Await the HTTP server task and handle it separately
    let http_result = http_server.await;
    if let Err(e) = http_result {
        eprintln!("HTTP server task failed: {:?}", e);
    }

    Ok(())
}
