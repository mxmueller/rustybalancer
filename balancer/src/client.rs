use std::sync::Arc;
use std::fmt;
use hyper::{Client, client::HttpConnector, Request, Response, Body};
use hyper_tls::HttpsConnector;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};
use futures::future::join_all;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub enum ClientError {
    RequestCanceled,
    RequestTimeout,
    HyperError(hyper::Error),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::RequestCanceled => write!(f, "Request was canceled"),
            ClientError::RequestTimeout => write!(f, "Request timed out"),
            ClientError::HyperError(e) => write!(f, "Hyper error: {}", e),
        }
    }
}

impl From<hyper::Error> for ClientError {
    fn from(err: hyper::Error) -> ClientError {
        ClientError::HyperError(err)
    }
}

struct QueuedRequest {
    request: Request<Body>,
    response_sender: mpsc::Sender<Result<Response<Body>, ClientError>>,
}

pub struct UnboundedClient {
    client: Client<HttpsConnector<HttpConnector>>,
    request_sender: mpsc::Sender<QueuedRequest>,
}

impl UnboundedClient {
    pub fn new() -> Arc<Self> {
        let https = HttpsConnector::new();
        let client = Client::builder()
            .pool_idle_timeout(Some(Duration::from_secs(30)))
            .build::<_, hyper::Body>(https);

        let (request_sender, mut request_receiver) = mpsc::channel::<QueuedRequest>(100_000);

        let client_clone = client.clone();
        tokio::spawn(async move {
            while let Some(queued_request) = request_receiver.recv().await {
                let client = client_clone.clone();
                tokio::spawn(async move {
                    let result = client.request(queued_request.request).await
                        .map_err(ClientError::from);
                    match &result {
                        Ok(response) => println!("Request successful. Status: {:?}", response.status()),
                        Err(e) => println!("Request failed: {}", e),
                    }
                    if let Err(e) = queued_request.response_sender.send(result).await {
                        println!("Failed to send response: {}", e);
                    }
                });
            }
        });

        Arc::new(UnboundedClient {
            client,
            request_sender,
        })
    }

    pub async fn request(&self, request: Request<Body>) -> Result<Response<Body>, ClientError> {
        let (response_sender, mut response_receiver) = mpsc::channel(1);
        let queued_request = QueuedRequest {
            request,
            response_sender,
        };

        if let Err(e) = self.request_sender.send(queued_request).await {
            println!("Failed to queue request: {}", e);
            return Err(ClientError::RequestCanceled);
        }
        match timeout(REQUEST_TIMEOUT, response_receiver.recv()).await {
            Ok(Some(result)) => {
                println!("Received response within timeout");
                result
            },
            Ok(None) => {
                println!("Channel closed unexpectedly");
                Err(ClientError::RequestCanceled)
            },
            Err(_) => {
                println!("Request timed out");
                Err(ClientError::RequestTimeout)
            }
        }
    }
}

// Keep this function for compatibility
pub fn spawn_workers<F>(num_workers: usize, work: F)
where
    F: Fn() -> tokio::task::JoinHandle<()> + Send + Sync + 'static,
{
    let work = Arc::new(work);
    for i in 0..num_workers {
        let work = work.clone();
        tokio::spawn(async move {
            println!("Worker {} started", i);
            loop {
                let handle = work();
                handle.await.unwrap();
            }
        });
    }
}