use tokio;
use reqwest;
use std::env;
use std::time::{Duration, Instant};
use tokio::time::interval;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <base_url> <requests_per_second> <test_duration_seconds> <intensity>", args[0]);
        std::process::exit(1);
    }

    let base_url = args[1].clone();
    let requests_per_second: u64 = args[2].parse()?;
    let test_duration_seconds: u64 = args[3].parse()?;
    let intensity: u64 = args[4].parse()?;

    println!("Starting load test on {} for {} seconds with {} requests per second and intensity {}",
             base_url, test_duration_seconds, requests_per_second, intensity);

    let client = Arc::new(reqwest::Client::new());
    let start_time = Instant::now();
    let total_requests = Arc::new(AtomicUsize::new(0));

    let interval_duration = Duration::from_micros(1_000_000 / requests_per_second);
    let mut interval = interval(interval_duration);

    while start_time.elapsed() < Duration::from_secs(test_duration_seconds) {
        interval.tick().await;

        let cpu_url = format!("{}/consume_cpu/{}", base_url, intensity);
        let memory_url = format!("{}/consume_memory/{}", base_url, intensity);
        let client_clone = Arc::clone(&client);
        let total_requests_clone = Arc::clone(&total_requests);

        tokio::spawn(async move {
            // CPU consumption request
            match client_clone.get(&cpu_url).send().await {
                Ok(response) => println!("CPU Consumption Response: {:?}", response.status()),
                Err(e) => println!("CPU Consumption Request Error: {:?}", e),
            }
            total_requests_clone.fetch_add(1, Ordering::SeqCst);

            // Memory consumption request
            match client_clone.get(&memory_url).send().await {
                Ok(response) => println!("Memory Consumption Response: {:?}", response.status()),
                Err(e) => println!("Memory Consumption Request Error: {:?}", e),
            }
            total_requests_clone.fetch_add(1, Ordering::SeqCst);
        });
    }

    // Wait a bit for any pending requests to complete
    tokio::time::sleep(Duration::from_secs(1)).await;

    let total_elapsed = start_time.elapsed().as_secs_f64();
    let total_requests = total_requests.load(Ordering::SeqCst);
    println!("Load test completed. Total requests: {}, Average rate: {:.2} req/s",
             total_requests, total_requests as f64 / total_elapsed);

    Ok(())
}