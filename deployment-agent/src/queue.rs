use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::container::manage_containers;
use crate::stats::get_container_statuses;
use std::env;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::future::Future;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueItem {
     pub(crate) name: String,
     pub(crate) external_port: String,
     pub(crate) score: f64,
     pub(crate) utilization_category: String,
}

pub type SharedQueue = Arc<Mutex<VecDeque<QueueItem>>>;

pub fn build_queue() -> Pin<Box<dyn Future<Output = Result<SharedQueue, axum::http::StatusCode>> + Send>> {
     Box::pin(async move {
          let queue = VecDeque::new();
          let shared_queue = Arc::new(Mutex::new(queue));

          let app_identifier = env::var("APP_IDENTIFIER").expect("APP_IDENTIFIER must be set");
          let default_container: i16 = env::var("DEFAULT_CONTAINER")
              .expect("DEFAULT_CONTAINER must be set")
              .parse()
              .expect("DEFAULT_CONTAINER must be a valid number");

          match manage_containers(&app_identifier, default_container).await {
               Ok(mut managed_containers) => {
                    match get_container_statuses().await {
                         Ok(container_statuses) => {
                              // Update scores for managed containers
                              for managed_container in &mut managed_containers {
                                   if let Some(status) = container_statuses.iter().find(|s| s.name == managed_container.name) {
                                        managed_container.score = status.overall_score;
                                        managed_container.utilization_category = status.utilization_category.clone();
                                   }
                              }

                              // Sort managed_containers by score in descending order
                              managed_containers.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

                              let mut locked_queue = shared_queue.lock().await;
                              for container in managed_containers {
                                   locked_queue.push_back(container);
                              }

                              // Print the queue
                              println!("Current Queue:");
                              for (index, item) in locked_queue.iter().enumerate() {
                                   println!("  {}. {} (Port: {}, Score: {:.2}, Category: {})",
                                            index + 1, item.name, item.external_port, item.score, item.utilization_category);
                              }

                              if locked_queue.is_empty() {
                                   println!("Queue is empty after rebuild. Triggering immediate rebuild...");
                                   drop(locked_queue);
                                   return build_queue().await;
                              }
                         }
                         Err(e) => {
                              eprintln!("Failed to get container statuses: {:?}", e);
                              return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
                         }
                    }
               }
               Err(e) => {
                    eprintln!("Failed to manage containers: {:?}", e);
                    return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
               }
          }

          Ok(shared_queue)
     })
}