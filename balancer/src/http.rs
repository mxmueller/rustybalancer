use std::env;
use std::sync::Arc;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
use crate::socket::SharedState;
use tokio::sync::Mutex;

async fn handle_request(req: Request<Body>, shared_state: SharedState) -> Result<Response<Body>, hyper::Error> {
    let state = shared_state.lock().await;
    if let Some(queue_items) = &*state {
        // Simple round-robin load balancing
        let backend_index = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as usize) % queue_items.len();
        let backend = &queue_items[backend_index];
        let host_internal = env::var("HOST_IP_HOST_INTERNAL").expect("HOST_IP_HOST_INTERNAL must be set");

        let uri_string = format!("http://{}:{}{}", host_internal, backend.external_port, req.uri().path());
        let uri: Uri = uri_string.parse().unwrap();

        let (mut parts, body) = req.into_parts();
        parts.uri = uri;
        let new_req = Request::from_parts(parts, body);

        println!("Forwarding request to worker: {}", backend.name);  // Print the worker handling the request

        let client = Client::new();
        return client.request(new_req).await;
    }

    Ok(Response::builder()
        .status(500)
        .body(Body::from("No backend available"))
        .unwrap())
}

pub async fn start_http_server(shared_state: SharedState) -> Result<(), Box<dyn std::error::Error>> {
    let addr = ([0, 0, 0, 0], env::var("HOST_PORT_HTTP_BALANCER").unwrap().parse().unwrap()).into();

    let make_svc = make_service_fn(move |_| {
        let shared_state = shared_state.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_request(req, shared_state.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
