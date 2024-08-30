use std::sync::Arc;
use hyper::{Client, client::HttpConnector};
use hyper_tls::HttpsConnector;
use tokio::sync::{Semaphore, Mutex};
use std::time::Duration;
use log::{info, warn};
use futures::future::join_all;

const MAX_CONNECTIONS: usize = 10_000;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_IDLE_CONNECTIONS: usize = 1_000;

#[derive(Debug)]
pub enum ClientError {
    SemaphoreError,
    HyperError(hyper::Error),
}

impl From<hyper::Error> for ClientError {
    fn from(err: hyper::Error) -> Self {
        ClientError::HyperError(err)
    }
}

pub struct SharedClient {
    clients: Vec<Client<HttpsConnector<HttpConnector>>>,
    semaphore: Arc<Semaphore>,
    current_client: Mutex<usize>,
}

impl SharedClient {
    pub fn new() -> Arc<Self> {
        info!("Initializing SharedClient with {} max connections", MAX_CONNECTIONS);

        let mut http = HttpConnector::new();
        http.set_nodelay(true);
        http.set_keepalive(Some(CONNECTION_TIMEOUT));
        let https = HttpsConnector::new();

        let num_workers = num_cpus::get();
        let clients: Vec<Client<HttpsConnector<HttpConnector>>> = (0..num_workers)
            .map(|_| {
                Client::builder()
                    .pool_max_idle_per_host(MAX_IDLE_CONNECTIONS / num_workers)
                    .pool_idle_timeout(Some(CONNECTION_TIMEOUT))
                    .build::<_, hyper::Body>(https.clone())
            })
            .collect();

        Arc::new(SharedClient {
            clients,
            semaphore: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
            current_client: Mutex::new(0),
        })
    }

    pub async fn get(&self) -> Result<ClientGuard<'_>, ClientError> {
        match self.semaphore.clone().acquire_owned().await {
            Ok(permit) => {
                let mut current = self.current_client.lock().await;
                let client_index = *current;
                *current = (*current + 1) % self.clients.len();

                Ok(ClientGuard {
                    shared_client: self,
                    client_index,
                    _permit: permit,
                })
            },
            Err(_) => {
                warn!("Failed to acquire semaphore permit");
                Err(ClientError::SemaphoreError)
            }
        }
    }

    pub async fn execute_requests<F, Fut, T>(&self, requests: Vec<F>) -> Vec<Result<T, ClientError>>
    where
        F: for<'a> FnOnce(ClientGuard<'a>) -> Fut,
        Fut: std::future::Future<Output = Result<T, hyper::Error>>,
    {
        let futures = requests.into_iter().map(|request| {
            let client = self;
            async move {
                match client.get().await {
                    Ok(guard) => request(guard).await.map_err(ClientError::from),
                    Err(e) => Err(e),
                }
            }
        });

        join_all(futures).await
    }
}

pub struct ClientGuard<'a> {
    shared_client: &'a SharedClient,
    client_index: usize,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl<'a> std::ops::Deref for ClientGuard<'a> {
    type Target = Client<HttpsConnector<HttpConnector>>;

    fn deref(&self) -> &Self::Target {
        &self.shared_client.clients[self.client_index]
    }
}

// Helper function to create a pool of worker threads
pub fn spawn_workers<F>(num_workers: usize, work: F)
where
    F: Fn() -> tokio::task::JoinHandle<()> + Send + Sync + 'static,
{
    let work = Arc::new(work);
    for _ in 0..num_workers {
        let work = work.clone();
        tokio::spawn(async move {
            loop {
                let handle = work();
                handle.await.unwrap();
            }
        });
    }
}