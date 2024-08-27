use reqwest;
use tokio;
use rand::Rng;
use std::time::{Duration, Instant};
use std::sync::Arc;
use futures::future::join_all;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::f64::consts::PI;

static TASK_COUNTER: AtomicUsize = AtomicUsize::new(0);

async fn send_request(client: Arc<reqwest::Client>, url: String, task_id: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = client.get(&url).send().await?;
    println!("Task {} response status for {}: {}", task_id, url, response.status());
    Ok(())
}

fn calculate_requests(time: f64, min_requests: u64, max_requests: u64) -> u64 {
    let cycle_duration = 300.0; // 5 minutes per full cycle
    let phase = 2.0 * PI * (time / cycle_duration);
    let sin_value = f64::sin(phase);
    let normalized = (sin_value + 1.0) / 2.0; // Normalize to 0-1 range
    (min_requests as f64 + (max_requests - min_requests) as f64 * normalized) as u64
}

async fn run_load_generator(client: Arc<reqwest::Client>, base_url: String, thread_id: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let start_time = Instant::now();
    let max_duration = Duration::from_secs(3600); // 1 hour max runtime

    let endpoints = [
        "/",
        "/consume",
        "/status",
    ];

    while start_time.elapsed() < max_duration {
        let elapsed_seconds = start_time.elapsed().as_secs_f64();
        let num_requests = calculate_requests(elapsed_seconds, 1, 100);

        let mut tasks = Vec::new();

        for _ in 0..num_requests {
            let endpoint = endpoints[rand::thread_rng().gen_range(0..endpoints.len())];
            let url = format!("{}{}", base_url, endpoint);
            let task_id = TASK_COUNTER.fetch_add(1, Ordering::SeqCst);

            let client_clone = Arc::clone(&client);
            tasks.push(tokio::spawn(async move {
                if let Err(e) = send_request(client_clone, url, task_id).await {
                    eprintln!("Error in request: {:?}", e);
                }
            }));
        }

        // Wait for all requests to complete
        join_all(tasks).await;

        // Dynamic delay based on the current number of requests
        let delay = 1000 / num_requests.max(1); // Ensure we don't divide by zero
        tokio::time::sleep(Duration::from_millis(delay)).await;

        println!("Thread {}: Sent {} requests", thread_id, num_requests);
    }

    println!("Thread {} finished after reaching max duration", thread_id);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(reqwest::Client::new());
    let base_url = "http://localhost:2548".to_string();  // Adjust this to your Docker container's address

    let mut handles = vec![];

    // Using 8 threads
    for thread_id in 0..8 {
        let client_clone = Arc::clone(&client);
        let base_url_clone = base_url.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = run_load_generator(client_clone, base_url_clone, thread_id).await {
                eprintln!("Error in thread {}: {:?}", thread_id, e);
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    join_all(handles).await;

    println!("Load test completed.");
    Ok(())
}