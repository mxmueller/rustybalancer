use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use futures::future::join_all;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, Uri};
use tokio::sync::{RwLock, Semaphore};
use rand::Rng;

use crate::queue::QueueItem;
use crate::socket::SharedState;
use crate::pool::ConnectionPool;

struct DynamicWeightedBalancer {
    last_update: Instant,
    queue_items: Vec<QueueItem>,
    cached_weights: Vec<f64>,
    total_weight: f64,
    update_interval: Duration,
}

impl DynamicWeightedBalancer {
    fn new(queue_items: Vec<QueueItem>) -> Self {
        let mut balancer = Self {
            last_update: Instant::now(),
            queue_items,
            cached_weights: Vec::new(),
            total_weight: 0.0,
            update_interval: Duration::from_secs(10),
        };
        balancer.update_weights();
        balancer
    }

    fn update_weights(&mut self) {
        self.cached_weights.clear();
        self.total_weight = 0.0;
        for item in &self.queue_items {
            let weight = 1.0 / item.score;
            self.cached_weights.push(weight);
            self.total_weight += weight;
        }
        self.last_update = Instant::now();
    }

    fn next(&mut self) -> Option<&QueueItem> {
        if self.queue_items.is_empty() {
            return None;
        }

        if self.last_update.elapsed() >= self.update_interval {
            self.update_weights();
        }

        let mut rng = rand::thread_rng();
        let random_point = rng.gen::<f64>() * self.total_weight;

        let mut cumulative = 0.0;
        for (index, weight) in self.cached_weights.iter().enumerate() {
            cumulative += weight;
            if cumulative >= random_point {
                return Some(&self.queue_items[index]);
            }
        }

        self.queue_items.first()
    }

    fn print_queue(&self) {
        println!("Current Queue in Balancer:");
        for (index, item) in self.queue_items.iter().enumerate() {
            println!("  {}. {} (Port: {}, Score: {:.2}, Category: {})",
                     index + 1, item.name, item.external_port, item.score, item.utilization_category);
        }
    }
}

async fn handle_request(
    req: Request<Body>,
    shared_state: SharedState,
    balancer: Arc<RwLock<DynamicWeightedBalancer>>,
    pool: Arc<ConnectionPool>,
    semaphore: Arc<Semaphore>,
) -> Result<Response<Body>, hyper::Error> {
    let _permit = semaphore.acquire().await.unwrap();

    let state = shared_state.read().await;
    if let Some(queue_items) = &*state {
        let mut balancer = balancer.write().await;

        if queue_items != &balancer.queue_items {
            println!("Updating queue items in balancer");
            *balancer = DynamicWeightedBalancer::new(queue_items.clone());
        }

        balancer.print_queue();

        if let Some(item) = balancer.next() {
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
    } else {
        println!("No queue items available in shared state");
    }

    Ok(Response::builder()
        .status(500)
        .body(Body::from("No backend available"))
        .unwrap())
}

pub async fn start_http_server(shared_state: SharedState, connection_pool: Arc<ConnectionPool>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = ([0, 0, 0, 0], env::var("HOST_PORT_HTTP_BALANCER").unwrap().parse().unwrap()).into();

    let balancer = Arc::new(RwLock::new(DynamicWeightedBalancer::new(vec![])));

    // Create a semaphore to limit concurrent requests
    let max_concurrent_requests = 1000; // Adjust this value based on your system's capabilities
    let semaphore = Arc::new(Semaphore::new(max_concurrent_requests));

    let make_svc = make_service_fn(move |_| {
        let shared_state = shared_state.clone();
        let balancer = balancer.clone();
        let pool = connection_pool.clone();
        let semaphore = semaphore.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_request(req, shared_state.clone(), balancer.clone(), pool.clone(), semaphore.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}