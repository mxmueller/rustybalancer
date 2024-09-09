use tokio;
use reqwest;
use std::env;
use std::time::{Duration, Instant};
use tokio::time::interval;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

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

    println!("Starting load test on {} for {} seconds with target {} requests per second and intensity {}",
             base_url, test_duration_seconds, requests_per_second, intensity);

    let client = Arc::new(reqwest::Client::new());
    let start_time = Instant::now();
    let total_requests = Arc::new(AtomicUsize::new(0));
    let successful_requests = Arc::new(AtomicUsize::new(0));
    let failed_requests = Arc::new(AtomicUsize::new(0));

    // Clone for the statistics thread
    let total_requests_clone = Arc::clone(&total_requests);
    let successful_requests_clone = Arc::clone(&successful_requests);
    let failed_requests_clone = Arc::clone(&failed_requests);
    let start_time_clone = start_time;

    // Spawn a thread to print statistics
    thread::spawn(move || {
        let mut last_total = 0;
        let mut last_time = start_time_clone;
        loop {
            thread::sleep(Duration::from_secs(1));
            let now = Instant::now();
            let current_total = total_requests_clone.load(Ordering::Relaxed);
            let current_successful = successful_requests_clone.load(Ordering::Relaxed);
            let current_failed = failed_requests_clone.load(Ordering::Relaxed);
            let duration = now.duration_since(last_time).as_secs_f64();
            let requests_per_second = (current_total - last_total) as f64 / duration;

            println!("Current rate: {:.2} req/s | Total: {} | Successful: {} | Failed: {} | Elapsed: {:.2}s",
                     requests_per_second, current_total, current_successful, current_failed,
                     now.duration_since(start_time_clone).as_secs_f64());

            last_total = current_total;
            last_time = now;

            if now.duration_since(start_time_clone) > Duration::from_secs(test_duration_seconds) {
                break;
            }
        }
    });

    let interval_duration = Duration::from_micros(1_000_000 / requests_per_second);
    let mut interval = interval(interval_duration);

    while start_time.elapsed() < Duration::from_secs(test_duration_seconds) {
        interval.tick().await;

        let client_clone = Arc::clone(&client);
        let total_requests_clone = Arc::clone(&total_requests);
        let successful_requests_clone = Arc::clone(&successful_requests);
        let failed_requests_clone = Arc::clone(&failed_requests);
        let base_url_clone = base_url.clone();

        tokio::spawn(async move {
            if intensity == 0 {
                // Send a GET request to the root path
                match client_clone.get(&base_url_clone).send().await {
                    Ok(_) => { successful_requests_clone.fetch_add(1, Ordering::Relaxed); }
                    Err(_) => { failed_requests_clone.fetch_add(1, Ordering::Relaxed); }
                };
                total_requests_clone.fetch_add(1, Ordering::Relaxed);
            } else {
                // Existing CPU and memory consumption logic
                let cpu_url = format!("{}/consume_cpu/{}", base_url_clone, intensity);
                let memory_url = format!("{}/consume_memory/{}", base_url_clone, intensity);

                // CPU consumption request
                match client_clone.get(&cpu_url).send().await {
                    Ok(_) => { successful_requests_clone.fetch_add(1, Ordering::Relaxed); }
                    Err(_) => { failed_requests_clone.fetch_add(1, Ordering::Relaxed); }
                };
                total_requests_clone.fetch_add(1, Ordering::Relaxed);

                // Memory consumption request
                match client_clone.get(&memory_url).send().await {
                    Ok(_) => { successful_requests_clone.fetch_add(1, Ordering::Relaxed); }
                    Err(_) => { failed_requests_clone.fetch_add(1, Ordering::Relaxed); }
                };
                total_requests_clone.fetch_add(1, Ordering::Relaxed);
            }
        });
    }

    // Wait a bit for any pending requests to complete
    tokio::time::sleep(Duration::from_secs(1)).await;

    let total_elapsed = start_time.elapsed().as_secs_f64();
    let total_requests = total_requests.load(Ordering::Relaxed);
    let successful_requests = successful_requests.load(Ordering::Relaxed);
    let failed_requests = failed_requests.load(Ordering::Relaxed);
    println!("\nLoad test completed.");
    println!("Total requests: {}", total_requests);
    println!("Successful requests: {}", successful_requests);
    println!("Failed requests: {}", failed_requests);
    println!("Average rate: {:.2} req/s", total_requests as f64 / total_elapsed);
    println!("Test duration: {:.2} seconds", total_elapsed);

    Ok(())
}