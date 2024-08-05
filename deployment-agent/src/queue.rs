use std::collections::VecDeque;
use std::sync::Arc;
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::stats::{get_container_status, ContainerStatus};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueItem {
     name: String,
     external_port: String,
}

pub type SharedQueue = Arc<Mutex<VecDeque<QueueItem>>>;

pub async fn build_queue() -> Result<SharedQueue, axum::http::StatusCode> {
     let queue = VecDeque::new();
     let shared_queue = Arc::new(Mutex::new(queue));

     // Fetching container status
     match get_container_status().await {
          Ok(stats) => {
               let mut queue = shared_queue.lock().await;
               for container in stats {
                    if let Some(ports) = container.ports.get("80/tcp") {
                         if let Some(external_port) = ports.get(0) {
                              // Extract only the port part after the colon
                              if let Some(port) = external_port.split(':').last() {
                                   let item = QueueItem {
                                        name: container.name.clone(),
                                        external_port: port.to_string(),
                                   };
                                   queue.push_back(item);
                              }
                         }
                    }
               }
          }
          Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
     }

     Ok(shared_queue)
}

pub async fn enqueue(queue: SharedQueue, item: QueueItem) {
     let mut queue = queue.lock().await;
     queue.push_back(item);
}

pub async fn dequeue(queue: SharedQueue) -> Option<QueueItem> {
     let mut queue = queue.lock().await;
     queue.pop_front()
}

pub async fn read_queue(queue: SharedQueue) -> Vec<QueueItem> {
     let queue = queue.lock().await;
     queue.iter().cloned().collect()
}

pub async fn display(queue: SharedQueue) {
     let queue = queue.lock().await;
     for item in queue.iter() {
          println!("{:?}", item);
     }
}
