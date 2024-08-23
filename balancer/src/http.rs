use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
use tokio::sync::RwLock;
use rand::Rng;

use crate::queue::QueueItem;
use crate::socket::SharedState;

struct DynamicWeightedBalancer {
    last_update: Instant,
    queue_items: Vec<QueueItem>,
}

impl DynamicWeightedBalancer {
    fn new(queue_items: Vec<QueueItem>) -> Self {
        Self {
            last_update: Instant::now(),
            queue_items,
        }
    }

    fn next(&self) -> Option<&QueueItem> {
        if self.queue_items.is_empty() {
            return None;
        }

        let total_inverse_score: f64 = self.queue_items.iter().map(|item| 1.0 / item.score).sum();
        let mut rng = rand::thread_rng();
        let random_point = rng.gen::<f64>() * total_inverse_score;

        let mut cumulative = 0.0;
        for item in &self.queue_items {
            cumulative += 1.0 / item.score;
            if cumulative >= random_point {
                return Some(item);
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
    balancer: Arc<RwLock<DynamicWeightedBalancer>>
) -> Result<Response<Body>, hyper::Error> {
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

            let client = Client::new();
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

pub async fn start_http_server(shared_state: SharedState) -> Result<(), Box<dyn std::error::Error>> {
    let addr = ([0, 0, 0, 0], env::var("HOST_PORT_HTTP_BALANCER").unwrap().parse().unwrap()).into();

    let balancer = Arc::new(RwLock::new(DynamicWeightedBalancer::new(vec![])));

    let make_svc = make_service_fn(move |_| {
        let shared_state = shared_state.clone();
        let balancer = balancer.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_request(req, shared_state.clone(), balancer.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}