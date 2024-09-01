use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, Uri};
use tokio::sync::{RwLock, Mutex};
use rand::distributions::{WeightedIndex, Distribution};
use rand::Rng;
use tokio::time::{interval, sleep, timeout};
use log::{info, error, warn};
use std::net::ToSocketAddrs;

use crate::queue::QueueItem;
use crate::socket::SharedState;
use crate::client::{SharedClient, spawn_workers};
use crate::cache::SimpleCache;

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
                .filter(|item| item.utilization_category != "SUNDOWN")
                .map(|item| WeightedQueueItem {
                    weight: Self::calculate_weight(item.score),
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

    fn calculate_weight(score: f64) -> f64 {
        if score < 0.0 || score > 100.0 {
            warn!("Invalid score: {}. Using default weight.", score);
            1.0 // default weight
        } else {
            score
        }
    }

    async fn update_weights(&self) {
        let mut last_update = self.last_update.lock().await;
        if last_update.elapsed() >= self.update_interval {
            let mut items = self.items.write().await;
            items.retain(|item| item.item.utilization_category != "SUNDOWN");
            for item in items.iter_mut() {
                item.weight = Self::calculate_weight(item.item.score);
                if item.weight == 0.0 {
                    warn!("Item {} has a weight of 0 (score: {})", item.item.name, item.item.score);
                }
            }
            *last_update = Instant::now();
        }
    }

    async fn next(&self) -> Option<QueueItem> {
        let items = self.items.read().await;
        if items.is_empty() {
            return None;
        }

        let weights: Vec<f64> = items.iter().map(|item| item.weight.max(f64::EPSILON)).collect();
        if weights.iter().all(|&w| w == 0.0) {
            warn!("All weights are zero. Selecting a random item.");
            let index = rand::thread_rng().gen_range(0..items.len());
            return Some(items[index].item.clone());
        }

        match WeightedIndex::new(&weights) {
            Ok(dist) => {
                let mut rng = rand::thread_rng();
                let chosen_index = dist.sample(&mut rng);
                Some(items[chosen_index].item.clone())
            },
            Err(e) => {
                error!("Failed to create WeightedIndex: {}. Selecting a random item.", e);
                let index = rand::thread_rng().gen_range(0..items.len());
                Some(items[index].item.clone())
            }
        }
    }

    async fn set_queue_items(&self, queue_items: Vec<QueueItem>) {
        let mut items = self.items.write().await;
        *items = queue_items
            .into_iter()
            .filter(|item| item.utilization_category != "SUNDOWN")
            .map(|item| WeightedQueueItem {
                weight: Self::calculate_weight(item.score),
                item,
            })
            .collect();
    }

    async fn print_queue(&self) {
        let items = self.items.read().await;
        info!("Current Queue in Balancer:");
        for (index, weighted_item) in items.iter().enumerate() {
            let item = &weighted_item.item;
            info!("  {}. {} (Port: {}, Score: {:.2}, Category: {}, Weight: {:.2})",
                     index + 1, item.name, item.external_port, item.score, item.utilization_category, weighted_item.weight);
        }
    }
}

fn is_static_resource(path: &str) -> bool {
    let static_extensions = [".jpg", ".jpeg", ".png", ".gif", ".css", ".js"];
    static_extensions.iter().any(|ext| path.ends_with(ext))
}

async fn resolve_host(host: &str, port: u16) -> Option<String> {
    let addr = format!("{}:{}", host, port);
    match addr.to_socket_addrs() {
        Ok(mut addrs) => addrs.next().map(|addr| addr.ip().to_string()),
        Err(_) => None,
    }
}

async fn handle_request(
    req: Request<Body>,
    balancer: Arc<DynamicWeightedBalancer>,
    shared_client: Arc<SharedClient>,
    cache: Arc<SimpleCache>,
) -> Result<Response<Body>, hyper::Error> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let is_static = is_static_resource(&path);

    if is_static && method == hyper::Method::GET {
        let cache_key = uri.to_string();
        if let Some(cached_response) = cache.get(&cache_key).await {
            return Ok(Response::new(Body::from(cached_response)));
        }
    }

    if let Some(item) = balancer.next().await {
        let host_internal = env::var("HOST_IP_HOST_INTERNAL").expect("HOST_IP_HOST_INTERNAL must be set");
        let port = item.external_port.parse::<u16>().unwrap();

        info!("Forwarding request to worker: {} (Port: {}, Score: {:.2}, Category: {})",
                 item.name, item.external_port, item.score, item.utilization_category);

        let resolved_ip = resolve_host(&host_internal, port).await
            .unwrap_or_else(|| host_internal.clone());

        let uri_string = format!("http://{}:{}{}", resolved_ip, port, uri.path());
        let new_uri: Uri = uri_string.parse().unwrap();

        let mut retries = 0;
        let max_retries = 3;
        let mut delay = Duration::from_millis(100);

        loop {
            let req = Request::builder()
                .method(method.clone())
                .uri(new_uri.clone())
                .body(Body::empty())
                .unwrap();

            match shared_client.get().await {
                Ok(client) => {
                    match timeout(Duration::from_secs(30), client.request(req)).await {
                        Ok(Ok(response)) => {
                            if is_static && method == hyper::Method::GET && response.status().is_success() {
                                let (parts, body) = response.into_parts();
                                let body_bytes = hyper::body::to_bytes(body).await?;

                                let cache_key = uri.to_string();
                                cache.set(cache_key, body_bytes.to_vec(), Duration::from_secs(3600)).await;

                                return Ok(Response::from_parts(parts, Body::from(body_bytes)));
                            } else {
                                return Ok(response);
                            }
                        },
                        Ok(Err(e)) => {
                            error!("Request failed: {:?}", e);
                            if retries >= max_retries {
                                error!("Max retries reached");
                                return Ok(Response::builder()
                                    .status(503)
                                    .body(Body::from("Service Unavailable"))
                                    .unwrap());
                            }
                        },
                        Err(_) => {
                            error!("Request timed out");
                            if retries >= max_retries {
                                error!("Max retries reached");
                                return Ok(Response::builder()
                                    .status(504)
                                    .body(Body::from("Gateway Timeout"))
                                    .unwrap());
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to get client: {:?}", e);
                    if retries >= max_retries {
                        error!("Max retries reached");
                        return Ok(Response::builder()
                            .status(503)
                            .body(Body::from("Service Unavailable"))
                            .unwrap());
                    }
                }
            }

            retries += 1;
            sleep(delay).await;
            delay *= 2; // exponential backoff
        }
    } else {
        warn!("No backend available");
        Ok(Response::builder()
            .status(503)
            .body(Body::from("No backend available"))
            .unwrap())
    }
}

pub async fn start_http_server(
    shared_state: SharedState,
    shared_client: Arc<SharedClient>,
    cache: Arc<SimpleCache>
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = ([0, 0, 0, 0], env::var("HOST_PORT_HTTP_BALANCER").unwrap().parse().unwrap()).into();

    let balancer = Arc::new(DynamicWeightedBalancer::new(vec![]));

    let make_svc = make_service_fn({
        let balancer = balancer.clone();
        let client = shared_client.clone();
        let cache = cache.clone();
        move |_| {
            let balancer = balancer.clone();
            let client = client.clone();
            let cache = cache.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    handle_request(req, balancer.clone(), client.clone(), cache.clone())
                }))
            }
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening on http://{}", addr);

    let balancer_for_update = balancer.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            let state = shared_state.read().await;
            if let Some(queue_items) = &*state {
                balancer_for_update.set_queue_items(queue_items.clone()).await;
            }
            balancer_for_update.update_weights().await;
        }
    });

    let balancer_for_print = balancer.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            balancer_for_print.print_queue().await;
        }
    });

    server.await?;

    Ok(())
}