use std::collections::{VecDeque, HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::container::{manage_containers, generate_hash_based_key, update_container_category, create_single_container, remove_container, list_running_containers};
use crate::stats::{get_container_statuses, ContainerStatus};
use crate::db;
use std::env;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::future::Future;
use bollard::errors::Error as BollardError;
use redis::{Commands, RedisResult};
use std::time::{Duration, Instant};
use futures_util::SinkExt;
use once_cell::sync::Lazy;
use tabled::{Style, Table, Tabled};

const HIGH_LOAD_THRESHOLD: f64 = 55.0;
const CRITICAL_LOAD_THRESHOLD: f64 = 20.0;
const LOW_LOAD_THRESHOLD: f64 = 80.0;
const MAX_CONTAINERS: usize = 15;
const COOLDOWN_PERIOD: Duration = Duration::from_secs(5);
const SCALE_STEP: usize = 1;
const SCALE_CHECK_PERIOD: Duration = Duration::from_secs(10);

static GLOBAL_COOLDOWN: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));
static LAST_SCALE_CHECK: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));

#[derive(Serialize, Deserialize, Debug, Clone, Tabled)]
pub struct QueueItem {
     pub(crate) dns_name: String,
     pub(crate) score: f64,
     pub(crate) utilization_category: String,
}

const REQUIRED_FIELDS: [&str; 4] = ["category", "score", "port", "image"];

fn is_container_complete(fields: &HashMap<String, String>) -> bool {
     REQUIRED_FIELDS.iter().all(|&field| fields.contains_key(field))
}

pub type SharedQueue = Arc<Mutex<VecDeque<QueueItem>>>;

pub fn build_queue() -> Pin<Box<dyn Future<Output = Result<SharedQueue, axum::http::StatusCode>> + Send>> {
     Box::pin(async move {
          println!("Starting build_queue function");
          let queue = VecDeque::new();
          let shared_queue = Arc::new(Mutex::new(queue));

          let app_identifier = env::var("APP_IDENTIFIER").expect("APP_IDENTIFIER must be set");
          println!("App identifier: {}", app_identifier);

          let mut conn = db::get_redis_connection();
          let default_container: i16 = conn.get("DEFAULT_CONTAINER").unwrap_or_else(|_| {
               env::var("DEFAULT_CONTAINER")
                   .expect("DEFAULT_CONTAINER must be set")
                   .parse()
                   .expect("DEFAULT_CONTAINER must be a valid number")
          });
          println!("Default container count: {}", default_container);

          println!("Fetching running containers");
          let running_containers = match list_running_containers(&app_identifier).await {
               Ok(containers) => containers,
               Err(e) => {
                    eprintln!("Failed to list running containers: {:?}", e);
                    return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
               }
          };
          println!("Found {} running containers", running_containers.len());

          println!("Cleaning up database entries");
          let db_containers: Vec<String> = conn.keys("container:*")
              .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
          println!("Found {} container entries in database", db_containers.len());

          // Erstellen Sie einen HashSet der laufenden Container-Schlüssel
          let running_container_keys: HashSet<String> = running_containers.iter()
              .filter_map(|c| c.names.as_ref().and_then(|names| names.first()))
              .map(|name| generate_hash_based_key(&app_identifier, name.trim_start_matches('/')))
              .collect();

          for db_container in db_containers {
               if !running_container_keys.contains(&db_container) {
                    println!("Container with key {} is in the database but not running. Removing from database.", db_container);
                    let _: () = conn.del(&db_container)
                        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
               } else {
                    println!("Container with key {} is running and in the database.", db_container);
               }
          }

          let running_count = running_containers.len() as i16;
          println!("Current running containers: {}, Default containers: {}", running_count, default_container);
          if running_count < default_container {
               println!("Running containers ({}) less than DEFAULT_CONTAINER ({}). Starting new containers.", running_count, default_container);
               let containers_to_start = default_container - running_count;
               for i in 0..containers_to_start {
                    println!("Starting container {} of {}", i+1, containers_to_start);
                    let image_name = env::var("DOCKER_IMAGE").expect("DOCKER_IMAGE must be set");
                    let target_port: u16 = env::var("TARGET_PORT")
                        .expect("TARGET_PORT must be set")
                        .parse()
                        .expect("TARGET_PORT must be a valid number");

                    match create_single_container(&image_name, target_port, &app_identifier, &mut conn).await {
                         Ok(container) => println!("New container created successfully: {}", container.dns_name),
                         Err(e) => {
                              eprintln!("Failed to create new container: {:?}", e);
                              return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
                         }
                    }
               }
          }

          println!("Managing containers");
          match manage_containers(&app_identifier, default_container).await {
               Ok(mut managed_containers) => {
                    println!("Successfully managed containers. Count: {}", managed_containers.len());
                    match get_container_statuses().await {
                         Ok(container_statuses) => {
                              println!("Retrieved container statuses. Count: {}", container_statuses.len());
                              for managed_container in &mut managed_containers {
                                   let key = generate_hash_based_key(&app_identifier, &managed_container.dns_name);
                                   println!("Processing container: {} with key: {}", managed_container.dns_name, key);

                                   if let Ok(fields) = conn.hgetall::<_, HashMap<String, String>>(&key) {
                                        println!("Retrieved fields for container {}: {:?}", managed_container.dns_name, fields);
                                        if is_container_complete(&fields) {
                                             if let Some(status) = container_statuses.iter().find(|s| s.name.trim_start_matches('/') == managed_container.dns_name.trim_start_matches('/')) {
                                                  managed_container.score = status.overall_score;
                                                  managed_container.utilization_category = status.utilization_category.clone();

                                                  if let Err(e) = conn.hset::<_, _, _, ()>(&key, "score", managed_container.score.to_string()) {
                                                       eprintln!("Failed to update score in database for {}: {:?}", managed_container.dns_name, e);
                                                  }
                                                  if let Err(e) = update_container_category(&mut conn, &key, &managed_container.utilization_category) {
                                                       eprintln!("Failed to update category in database for {}: {:?}", managed_container.dns_name, e);
                                                  }

                                                  println!("Updated container {}: score = {}, category = {}",
                                                           managed_container.dns_name, managed_container.score, managed_container.utilization_category);
                                             } else {
                                                  println!("No status found for container: {}", managed_container.dns_name);
                                             }
                                        } else {
                                             println!("Container {} is incomplete. Fields: {:?}", managed_container.dns_name, fields);
                                        }
                                   } else {
                                        println!("Failed to retrieve container data for {}.", managed_container.dns_name);
                                   }
                              }

                              println!("Removing inactive SUNDOWN containers");
                              managed_containers = remove_inactive_sundown_containers(&app_identifier, managed_containers, &container_statuses).await;

                              println!("Checking and scaling containers");
                              if let Err(e) = check_and_scale_containers(&mut conn, &app_identifier, &container_statuses, &mut managed_containers).await {
                                   eprintln!("Failed to check and scale containers: {:?}", e);
                              }

                              println!("Sorting managed containers");
                              managed_containers.sort_by(|a, b| {
                                   if a.utilization_category == "SUNDOWN" && b.utilization_category != "SUNDOWN" {
                                        std::cmp::Ordering::Greater
                                   } else if a.utilization_category != "SUNDOWN" && b.utilization_category == "SUNDOWN" {
                                        std::cmp::Ordering::Less
                                   } else {
                                        b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
                                   }
                              });

                              println!("Updating shared queue");
                              let mut locked_queue = shared_queue.lock().await;
                              for container in managed_containers {
                                   locked_queue.push_back(container);
                              }

                              print_final_queue(&locked_queue);

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

          println!("build_queue function completed successfully");
          Ok(shared_queue)
     })
}
fn print_final_queue(queue: &VecDeque<QueueItem>) {
     let mut table = Table::new(queue);
     table.with(Style::modern());
     println!("{}", table);
}

async fn remove_inactive_sundown_containers(app_identifier: &str, containers: Vec<QueueItem>, container_statuses: &[ContainerStatus]) -> Vec<QueueItem> {
     let mut active_containers = Vec::new();
     let status_map: HashMap<String, &ContainerStatus> = container_statuses.iter()
         .map(|status| (status.name.trim_start_matches('/').to_string(), status))
         .collect();

     for container in containers {
          if container.utilization_category == "SUNDOWN" {
               let container_name = container.dns_name.trim_start_matches('/');
               if let Some(status) = status_map.get(container_name) {
                    if status.network_score >= 99.9 {
                         println!("Attempting to remove inactive SUNDOWN container: {}", container_name);
                         match remove_container(app_identifier, container_name).await {
                              Ok(_) => {
                                   println!("Successfully removed SUNDOWN container: {}", container_name);
                                   continue; // Skip adding this container to active_containers
                              }
                              Err(e) => {
                                   eprintln!("Failed to remove container {}: {:?}", container_name, e);
                                   active_containers.push(container);
                              }
                         }
                    } else {
                         println!("SUNDOWN container {} not removed. Network score: {}", container_name, status.network_score);
                         active_containers.push(container);
                    }
               } else {
                    println!("Status not found for SUNDOWN container {}. Attempting to remove it anyway.", container_name);
                    match remove_container(app_identifier, container_name).await {
                         Ok(_) => println!("Successfully removed SUNDOWN container without status: {}", container_name),
                         Err(e) => {
                              eprintln!("Failed to remove container without status {}: {:?}", container_name, e);
                              active_containers.push(container);
                         }
                    }
               }
          } else {
               active_containers.push(container);
          }
     }
     active_containers
}

async fn check_and_scale_containers(
     conn: &mut redis::Connection,
     app_identifier: &str,
     container_statuses: &[ContainerStatus],
     managed_containers: &mut [QueueItem],
) -> Result<(), BollardError> {
     println!("DEBUG: Entering check_and_scale_containers");
     let active_containers: Vec<_> = managed_containers.iter()
         .filter(|c| c.utilization_category != "SUNDOWN")
         .collect();

     let active_container_count = active_containers.len();
     let average_load = active_containers.iter()
         .map(|c| c.score)
         .sum::<f64>() / active_container_count as f64;

     let has_critically_loaded_container = active_containers.iter()
         .any(|c| c.score < CRITICAL_LOAD_THRESHOLD);

     println!("DEBUG: Active containers: {}, Average load: {:.2}, Has critically loaded container: {}",
              active_container_count, average_load, has_critically_loaded_container);

     let cooldown_status = get_cooldown_status().await;
     let can_scale = can_scale().await;

     let env_default_container: i16 = env::var("DEFAULT_CONTAINER")
         .unwrap_or_else(|_| "1".to_string())
         .parse()
         .expect("DEFAULT_CONTAINER must be a valid number");

     println!("Current conditions: Average load: {}, Active container count: {}, Has critically loaded container: {}, Cooldown: {}",
              average_load, active_container_count, has_critically_loaded_container, cooldown_status);

     let mut last_check = LAST_SCALE_CHECK.lock().await;
     if last_check.elapsed() >= SCALE_CHECK_PERIOD {
          *last_check = Instant::now();
          println!("Performing scale check...");

          if active_container_count >= MAX_CONTAINERS {
               println!("Max container limit ({}) reached. Cannot scale up further.", MAX_CONTAINERS);
          } else if can_scale && (average_load < HIGH_LOAD_THRESHOLD || has_critically_loaded_container) {
               let containers_to_add = std::cmp::min(SCALE_STEP, MAX_CONTAINERS - active_container_count);

               for _ in 0..containers_to_add {
                    let current_default: i16 = conn.get("DEFAULT_CONTAINER").unwrap_or(env_default_container);
                    let new_default = current_default + 1;

                    if let Err(e) = conn.set::<_, _, ()>("DEFAULT_CONTAINER", new_default) {
                         eprintln!("Failed to update DEFAULT_CONTAINER in Redis: {:?}", e);
                         return Err(BollardError::IOError {
                              err: std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                         });
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
               let current_default: i16 = conn.get("DEFAULT_CONTAINER").unwrap_or(env_default_container);

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
                         let key = generate_hash_based_key(app_identifier, &container.dns_name);
                         if let Err(e) = update_container_category(conn, &key, "SUNDOWN") {
                              eprintln!("Failed to mark container {} for SUNDOWN: {:?}", container.dns_name, e);
                         } else {
                              println!("Marked container {} for graceful shutdown", container.dns_name);
                              container.utilization_category = "SUNDOWN".to_string();
                         }
                    }

                    let new_default = std::cmp::max(current_default - containers_to_remove as i16, env_default_container);
                    if let Err(e) = conn.set::<_, _, ()>("DEFAULT_CONTAINER", new_default) {
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

     println!("DEBUG: Exiting check_and_scale_containers");
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