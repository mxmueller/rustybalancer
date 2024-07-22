use reqwest::Client;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;

async fn make_request(client: &Client, url: &str) {
    match client.get(url).send().await {
        Ok(_) => println!("Request successful: {}", url),
        Err(e) => eprintln!("Request error: {} - {:?}", url, e),
    }
}

async fn stress_test(url: &str, concurrency: usize, duration: Duration) {
    let client = Client::new();
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let start = tokio::time::Instant::now();

    loop {
        if start.elapsed() >= duration {
            break;
        }

        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client_clone = client.clone();
        let url = url.to_string();

        task::spawn(async move {
            make_request(&client_clone, &url).await;
            drop(permit);
        });

        sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn test_cpu_stress() {
    let cpu_url = "http://localhost:37259/cpu?duration=10";
    let concurrency = 3000; // Anzahl der gleichzeitigen Anfragen
    let test_duration = Duration::from_secs(20); // Dauer des Stresstests

    stress_test(cpu_url, concurrency, test_duration).await;
}

#[tokio::test]
async fn test_memory_stress() {
    let memory_url = "http://localhost:37259/memory?size=2";
    let concurrency = 80; // Anzahl der gleichzeitigen Anfragen
    let test_duration = Duration::from_secs(30); // Dauer des Stresstests

    stress_test(memory_url, concurrency, test_duration).await;
}
