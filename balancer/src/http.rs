use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use futures::future::join_all;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, Uri};
use tokio::sync::{RwLock, Mutex, Semaphore};
use rand::distributions::{WeightedIndex, Distribution};
use tokio::time::interval;

use crate::queue::QueueItem;
use crate::socket::SharedState;
use crate::pool::ConnectionPool;

struct WeightedQueueItem {
    item: QueueItem,
    weight: f64,
}

struct DynamicWeightedBalancer {
    items: Arc<RwLock<Vec<WeightedQueueItem>>>,
    last_update: Arc<Mutex<Instant>>,
    update_interval: Duration,
}

impl DynamicWeightedBalancer {
    fn new(queue_items: Vec<QueueItem>) -> Self {
        let items = Arc::new(RwLock::new(
            queue_items
                .into_iter()
                .map(|item| WeightedQueueItem {
                    weight: 1.0 / item.score,
                    item,
                })
                .collect()
        ));

        Self {
            items,
            last_update: Arc::new(Mutex::new(Instant::now())),
            update_interval: Duration::from_secs(10),
        }
    }

    async fn update_weights(&self) {
        let mut last_update = self.last_update.lock().await;
        if last_update.elapsed() >= self.update_interval {
            let mut items = self.items.write().await;
            for item in items.iter_mut() {
                item.weight = 1.0 / item.item.score;
            }
            *last_update = Instant::now();
        }
    }

    async fn next(&self) -> Option<QueueItem> {
        let items = self.items.read().await;
        if items.is_empty() {
            return None;
        }

        let weights: Vec<f64> = items.iter().map(|item| item.weight).collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        let mut rng = rand::thread_rng();
        let chosen_index = dist.sample(&mut rng);

        Some(items[chosen_index].item.clone())
    }

    async fn set_queue_items(&self, queue_items: Vec<QueueItem>) {
        let mut items = self.items.write().await;
        *items = queue_items
            .into_iter()
            .map(|item| WeightedQueueItem {
                weight: 1.0 / item.score,
                item,
            })
            .collect();
    }

    async fn print_queue(&self) {
        let items = self.items.read().await;
        println!("Current Queue in Balancer:");
        for (index, weighted_item) in items.iter().enumerate() {
            let item = &weighted_item.item;
            println!("  {}. {} (Port: {}, Score: {:.2}, Category: {}, Weight: {:.2})",
                     index + 1, item.name, item.external_port, item.score, item.utilization_category, weighted_item.weight);
        }
    }
}

async fn handle_request(
    req: Request<Body>,
    balancer: Arc<DynamicWeightedBalancer>,
    pool: Arc<ConnectionPool>,
    semaphore: Arc<Semaphore>,
) -> Result<Response<Body>, hyper::Error> {
    let _permit = semaphore.acquire().await.unwrap();

    if let Some(item) = balancer.next().await {
        let host_internal = env::var("HOST_IP_HOST_INTERNAL").expect("HOST_IP_HOST_INTERNAL must be set");

        println!("Forwarding request to worker: {} (Port: {}, Score: {:.2}, Category: {})",
                 item.name, item.external_port, item.score, item.utilization_category);

        let uri_string = format!("http://{}:{}{}", host_internal, item.external_port, req.uri().path());
        let uri: Uri = uri_string.parse().unwrap();

        let (mut parts, body) = req.into_parts();
        parts.uri = uri;
        let new_req = Request::from_parts(parts, body);

        let client = pool.get().await;
        return client.request(new_req).await;
    }

    Ok(Response::builder()
        .status(500)
        .body(Body::from("No backend available"))
        .unwrap())
}

pub async fn start_http_server(shared_state: SharedState, connection_pool: Arc<ConnectionPool>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = ([0, 0, 0, 0], env::var("HOST_PORT_HTTP_BALANCER").unwrap().parse().unwrap()).into();

    let balancer = Arc::new(DynamicWeightedBalancer::new(vec![]));

    // Create a semaphore to limit concurrent requests
    let max_concurrent_requests = 10000; // Adjust this value based on your system's capabilities
    let semaphore = Arc::new(Semaphore::new(max_concurrent_requests));

    let make_svc = make_service_fn({
        let balancer = balancer.clone();
        let pool = connection_pool.clone();
        let semaphore = semaphore.clone();
        move |_| {
            let balancer = balancer.clone();
            let pool = pool.clone();
            let semaphore = semaphore.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    handle_request(req, balancer.clone(), pool.clone(), semaphore.clone())
                }))
            }
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    // Update the balancer with the latest queue items
    let balancer_for_update = balancer.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(100)); // Check every 100ms
        loop {
            interval.tick().await;
            let state = shared_state.read().await;
            if let Some(queue_items) = &*state {
                balancer_for_update.set_queue_items(queue_items.clone()).await;
            }
            balancer_for_update.update_weights().await;
        }
    });

    // Periodically print the queue (for monitoring)
    let balancer_for_print = balancer.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60)); // Print every 60 seconds
        loop {
            interval.tick().await;
            balancer_for_print.print_queue().await;
        }
    });

    server.await?;

    Ok(())
}