use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::container::{manage_containers, generate_hash_based_key, update_container_category, create_single_container};
use crate::stats::{get_container_statuses, ContainerStatus};
use crate::db;
use std::env;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::future::Future;
use bollard::errors::Error as BollardError;
use redis::RedisError;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

const HIGH_LOAD_THRESHOLD: f64 = 49.0;
const CRITICAL_LOAD_THRESHOLD: f64 = 20.0;
const LOW_LOAD_THRESHOLD: f64 = 70.0;
const MAX_CONTAINERS: usize = 10;
const COOLDOWN_PERIOD: Duration = Duration::from_secs(45);
const SCALE_STEP: usize = 1;
const SCALE_CHECK_PERIOD: Duration = Duration::from_secs(30);

static GLOBAL_COOLDOWN: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));
static LAST_SCALE_CHECK: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));

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
          let mut conn = db::get_redis_connection();
          let default_container: i16 = db::get_config_value(&mut conn, "DEFAULT_CONTAINER")
              .unwrap_or_else(|| {
                   env::var("DEFAULT_CONTAINER")
                       .expect("DEFAULT_CONTAINER must be set")
                       .parse()
                       .expect("DEFAULT_CONTAINER must be a valid number")
              });

          match manage_containers(&app_identifier, default_container).await {
               Ok(mut managed_containers) => {
                    match get_container_statuses().await {
                         Ok(container_statuses) => {
                              for managed_container in &mut managed_containers {
                                   let key = generate_hash_based_key(&app_identifier, managed_container.external_port.parse().unwrap_or(0));

                                   let db_category: String = db::get_config_value(&mut conn, &format!("{}:category", key))
                                       .unwrap_or_else(|| "UNKNOWN".to_string());

                                   if db_category == "SUNDOWN" {
                                        managed_container.utilization_category = "SUNDOWN".to_string();
                                        println!("Container {} is marked as SUNDOWN in the database. Keeping SUNDOWN status.", managed_container.name);
                                   } else if let Some(status) = container_statuses.iter().find(|s| s.name == managed_container.name) {
                                        managed_container.score = status.overall_score;
                                        if managed_container.utilization_category != "SUNDOWN" {
                                             managed_container.utilization_category = status.utilization_category.clone();
                                             if let Err(e) = update_container_category(&mut conn, &key, &managed_container.utilization_category) {
                                                  eprintln!("Failed to update category in database: {:?}", e);
                                             }
                                        } else {
                                             println!("Container {} is marked as SUNDOWN in memory. Keeping SUNDOWN status.", managed_container.name);
                                        }
                                   }
                              }

                              if let Err(e) = check_and_scale_containers(&mut conn, &app_identifier, &container_statuses, &mut managed_containers).await {
                                   eprintln!("Failed to check and scale containers: {:?}", e);
                              }

                              managed_containers.sort_by(|a, b| {
                                   if a.utilization_category == "SUNDOWN" && b.utilization_category != "SUNDOWN" {
                                        std::cmp::Ordering::Greater
                                   } else if a.utilization_category != "SUNDOWN" && b.utilization_category == "SUNDOWN" {
                                        std::cmp::Ordering::Less
                                   } else {
                                        b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
                                   }
                              });

                              let mut locked_queue = shared_queue.lock().await;
                              for container in managed_containers {
                                   locked_queue.push_back(container);
                              }

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

async fn check_and_scale_containers(
     conn: &mut redis::Connection,
     app_identifier: &str,
     container_statuses: &[ContainerStatus],
     managed_containers: &mut [QueueItem],
) -> Result<(), BollardError> {
     let active_containers: Vec<_> = managed_containers.iter()
         .filter(|c| c.utilization_category != "SUNDOWN")
         .collect();

     let active_container_count = active_containers.len();
     let average_load = active_containers.iter()
         .map(|c| c.score)
         .sum::<f64>() / active_container_count as f64;

     let has_critically_loaded_container = active_containers.iter()
         .any(|c| c.score < CRITICAL_LOAD_THRESHOLD);

     let cooldown_status = get_cooldown_status().await;
     let can_scale = can_scale().await;

     let env_default_container: i16 = env::var("DEFAULT_CONTAINER")
         .unwrap_or_else(|_| "1".to_string())
         .parse()
         .expect("DEFAULT_CONTAINER must be a valid number");

     println!("Current conditions: Average load: {}, Active container count: {}, Has critically loaded container: {}, Cooldown: {}",
              average_load, active_container_count, has_critically_loaded_container, cooldown_status);

     // Check if it's time to consider scaling (both up and down)
     let mut last_check = LAST_SCALE_CHECK.lock().await;
     if last_check.elapsed() >= SCALE_CHECK_PERIOD {
          *last_check = Instant::now();
          println!("Performing scale check...");

          if active_container_count >= MAX_CONTAINERS {
               println!("Max container limit ({}) reached. Cannot scale up further.", MAX_CONTAINERS);
          } else if can_scale && (average_load < HIGH_LOAD_THRESHOLD || has_critically_loaded_container) {
               // Scale up logic
               let containers_to_add = std::cmp::min(SCALE_STEP, MAX_CONTAINERS - active_container_count);

               for _ in 0..containers_to_add {
                    let current_default: i16 = db::get_config_value(conn, "DEFAULT_CONTAINER").unwrap_or(env_default_container);
                    let new_default = current_default + 1;

                    match db::set_config_value(conn, "DEFAULT_CONTAINER", new_default) {
                         Ok(_) => {},
                         Err(e) => {
                              eprintln!("Failed to update DEFAULT_CONTAINER in Redis: {:?}", e);
                              return Err(BollardError::IOError {
                                   err: std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                              });
                         }
                    }

                    let image_name = env::var("DOCKER_IMAGE").expect("DOCKER_IMAGE must be set");
                    let target_port: u16 = env::var("TARGET_PORT")
                        .expect("TARGET_PORT must be set")
                        .parse()
                        .expect("TARGET_PORT must be a valid number");

                    match create_single_container(&image_name, target_port, app_identifier, conn).await {
                         Ok(_) => {
                              println!("Created new container due to high load. New DEFAULT_CONTAINER value: {}", new_default);
                         },
                         Err(e) => {
                              eprintln!("Failed to create new container: {:?}", e);
                              return Err(e);
                         }
                    }
               }

               update_cooldown().await;
               println!("Cooldown period activated. Next scaling possible after {:?}", COOLDOWN_PERIOD);
               println!("Added {} new container(s). New active container count: {}", containers_to_add, active_container_count + containers_to_add);
          } else if average_load > LOW_LOAD_THRESHOLD && active_container_count > env_default_container as usize {
               // Scale down logic
               let current_default: i16 = db::get_config_value(conn, "DEFAULT_CONTAINER").unwrap_or(env_default_container);

               let containers_to_remove = std::cmp::min(
                    SCALE_STEP,
                    std::cmp::min(
                         active_container_count - env_default_container as usize,
                         (current_default - env_default_container) as usize
                    )
               );

               if containers_to_remove > 0 {
                    let mut containers_to_sundown: Vec<_> = managed_containers.iter_mut()
                        .filter(|c| c.utilization_category != "SUNDOWN" && c.utilization_category != "INIT")
                        .collect();
                    containers_to_sundown.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

                    for container in containers_to_sundown.iter_mut().take(containers_to_remove) {
                         let key = generate_hash_based_key(app_identifier, container.external_port.parse().unwrap_or(0));
                         if let Err(e) = update_container_category(conn, &key, "SUNDOWN") {
                              eprintln!("Failed to mark container {} for SUNDOWN: {:?}", container.name, e);
                         } else {
                              println!("Marked container {} for graceful shutdown", container.name);
                              container.utilization_category = "SUNDOWN".to_string();
                         }
                    }

                    // Update DEFAULT_CONTAINER in Redis
                    let new_default = std::cmp::max(current_default - containers_to_remove as i16, env_default_container);
                    if let Err(e) = db::set_config_value(conn, "DEFAULT_CONTAINER", new_default) {
                         eprintln!("Failed to update DEFAULT_CONTAINER in Redis: {:?}", e);
                    } else {
                         println!("Updated DEFAULT_CONTAINER to {} due to scale-down", new_default);
                    }

                    println!("Marked {} container(s) for SUNDOWN. New active container count: {}", containers_to_remove, active_container_count - containers_to_remove);
               } else {
                    println!("Cannot scale down further. Minimum number of active containers reached.");
               }
          } else {
               println!("Current conditions do not require scaling.");
          }
     } else {
          println!("Skipping scale check. Next check in {:?}", SCALE_CHECK_PERIOD.checked_sub(last_check.elapsed()).unwrap_or(Duration::from_secs(0)));
     }

     Ok(())
}

async fn can_scale() -> bool {
     let cooldown = GLOBAL_COOLDOWN.lock().await;
     cooldown.elapsed() >= COOLDOWN_PERIOD
}

async fn update_cooldown() {
     let mut cooldown = GLOBAL_COOLDOWN.lock().await;
     *cooldown = Instant::now();
}

async fn get_cooldown_status() -> String {
     let cooldown = GLOBAL_COOLDOWN.lock().await;
     let elapsed = cooldown.elapsed();
     if elapsed >= COOLDOWN_PERIOD {
          "inactive".to_string()
     } else {
          format!("active, next scaling possible in {:?}", COOLDOWN_PERIOD.checked_sub(elapsed).unwrap_or(Duration::from_secs(0)))
     }
}