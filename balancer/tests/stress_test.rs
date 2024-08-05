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

    let mut handles = vec![];

    while start.elapsed() < duration {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client_clone = client.clone();
        let url = url.to_string();

        let handle = task::spawn(async move {
            make_request(&client_clone, &url).await;
            drop(permit);
        });

        handles.push(handle);

        // Ein kleiner Schlaf, um die CPU-Nutzung zu reduzieren und realistische Bedingungen zu simulieren
        sleep(Duration::from_millis(1)).await;
    }

    for handle in handles {
        let _ = handle.await;
    }
}

#[tokio::test]
async fn test_stress() {
    let url = "http://localhost:2548/";
    let concurrency = 10000; // Anzahl der gleichzeitigen Anfragen
    let test_duration = Duration::from_secs(60); // Dauer des Stresstests

    stress_test(url, concurrency, test_duration).await;
}
