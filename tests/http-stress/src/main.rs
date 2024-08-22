use reqwest::Client;
use tokio::time::{sleep, Duration, Instant};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;
use std::sync::atomic::{AtomicUsize, Ordering};

async fn make_request(client: &Client, url: &str, request_count: &Arc<AtomicUsize>) -> Result<(), reqwest::Error> {
    let response = client.get(url).send().await?;
    request_count.fetch_add(1, Ordering::Relaxed);
    println!("Status: {}, URL: {}", response.status(), url);
    Ok(())
}

async fn stress_test(url: &str, concurrency: usize, duration: Duration, delay: Duration) {
    let client = Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap();
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let start = Instant::now();
    let request_count = Arc::new(AtomicUsize::new(0));

    while start.elapsed() < duration {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client_clone = client.clone();
        let url = url.to_string();
        let request_count = Arc::clone(&request_count);

        task::spawn(async move {
            let _ = make_request(&client_clone, &url, &request_count).await;
            drop(permit);
        });

        sleep(delay).await;
    }

    // Warte kurz, damit ausstehende Anfragen abgeschlossen werden können
    sleep(Duration::from_secs(5)).await;

    let total_requests = request_count.load(Ordering::Relaxed);
    let elapsed = start.elapsed();
    println!("Total requests: {}", total_requests);
    println!("Requests per second: {:.2}", total_requests as f64 / elapsed.as_secs_f64());
}

#[tokio::main]
async fn main() {
    let url = "http://localhost:2548/";
    let concurrency = 50000; // Sehr hohe Anzahl gleichzeitiger Anfragen
    let test_duration = Duration::from_secs(180); // 3 Minuten Testdauer
    let delay = Duration::from_micros(10); // Sehr kurze Verzögerung zwischen Anfragen

    println!("Starting stress test...");
    println!("URL: {}", url);
    println!("Concurrency: {}", concurrency);
    println!("Duration: {} seconds", test_duration.as_secs());
    println!("Delay between requests: {} microseconds", delay.as_micros());

    stress_test(url, concurrency, test_duration, delay).await;

    println!("Stress test completed.");
}