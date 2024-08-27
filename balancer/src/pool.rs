use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use hyper::Client;

pub struct ConnectionPool {
    pool: Mutex<VecDeque<Client<hyper::client::HttpConnector>>>,
    semaphore: Arc<Semaphore>,
}

impl ConnectionPool {
    pub fn new(size: usize) -> Arc<Self> {
        let mut pool = VecDeque::with_capacity(size);
        for _ in 0..size {
            pool.push_back(Client::new());
        }

        Arc::new(ConnectionPool {
            pool: Mutex::new(pool),
            semaphore: Arc::new(Semaphore::new(size)),
        })
    }

    pub async fn get(self: &Arc<Self>) -> PooledConnection {
        let _permit = self.semaphore.acquire().await.unwrap();
        let client = loop {
            if let Some(client) = self.pool.lock().await.pop_front() {
                break client;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        };

        PooledConnection {
            client: Some(client),
            pool: Arc::clone(self),
        }
    }

    async fn return_connection(&self, client: Client<hyper::client::HttpConnector>) {
        self.pool.lock().await.push_back(client);
        self.semaphore.add_permits(1);
    }
}

pub struct PooledConnection {
    client: Option<Client<hyper::client::HttpConnector>>,
    pool: Arc<ConnectionPool>,
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.return_connection(client).await;
            });
        }
    }
}

impl std::ops::Deref for PooledConnection {
    type Target = Client<hyper::client::HttpConnector>;

    fn deref(&self) -> &Self::Target {
        self.client.as_ref().unwrap()
    }
}