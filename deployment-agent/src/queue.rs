use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::pin::Pin;
use futures::Future;
use tokio::time::{interval, Duration};
use crate::container::{list_running_containers, check_and_stop_container_if_not_in_db, cleanup_orphaned_db_entries, create_container, generate_hash_based_key, container};
use crate::db;
use std::env;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueItem {
     pub(crate) name: String,
     pub(crate) external_port: String,
}

pub type SharedQueue = Arc<Mutex<VecDeque<QueueItem>>>;

pub async fn build_queue() -> Result<SharedQueue, axum::http::StatusCode> {
     let queue = VecDeque::new();
     let shared_queue = Arc::new(Mutex::new(queue));

     let mut conn = db::get_redis_connection();
     let app_identifier = env::var("APP_IDENTIFIER").expect("APP_IDENTIFIER must be set");
     let image_name = env::var("DOCKER_IMAGE").expect("DOCKER_IMAGE must be set");
     let target_port: u16 = env::var("TARGET_PORT")
         .expect("TARGET_PORT must be set")
         .parse()
         .expect("TARGET_PORT must be a valid number");

     let default_container: i16 = match db::get_config_value::<i16>(&mut conn, "DEFAULT_CONTAINER") {
          Some(value) => value,
          None => {
               eprintln!("DEFAULT_CONTAINER value not found in Redis.");
               return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
          }
     };

     // Start the containers management logic before building the queue
     match container().await {
          Ok(_) => println!("Containers checked and started as necessary."),
          Err(e) => eprintln!("Failed to check or start containers: {:?}", e),
     }

     // Hole aktuelle Liste laufender Container
     let containers = list_running_containers(&app_identifier)
         .await
         .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

     let mut running_container_keys = HashMap::new();

     {
          let mut locked_queue = shared_queue.lock().await;
          for container in containers {
               if let Some(item) = check_and_stop_container_if_not_in_db(&container, &mut conn, &app_identifier)
                   .await
                   .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
               {
                    running_container_keys.insert(
                         generate_hash_based_key(&app_identifier, item.external_port.parse().unwrap_or(0)),
                         container.id.clone().unwrap_or_default(),
                    );
                    locked_queue.push_back(item);
               }
          }

          // Check if the queue is empty, and trigger a rebuild if necessary
          if locked_queue.is_empty() {
               println!("Queue is empty after rebuild. Triggering immediate rebuild...");
               drop(locked_queue);  // Unlock the queue before calling build_queue again
               return Box::pin(build_queue()).await; // Use Box::pin for recursive call
          }
     }

     // Bereinigen von verwaisten Einträgen
     cleanup_orphaned_db_entries(&mut conn, &running_container_keys)
         .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

     // Fehlende Container starten
     let running_containers_count = running_container_keys.len() as i16;
     if running_containers_count < default_container {
          for _ in running_containers_count + 1..=default_container {
               // Erzeuge eine neue UUID
               let uuid = Uuid::new_v4();

               // Verwende die ersten 8 Zeichen der UUID für den Container-Namen, um den Namen kurz zu halten
               let container_name = format!("worker-{}", &uuid.to_string()[..8]);

               match create_container(&container_name, &image_name, target_port, &app_identifier, &mut conn).await {
                    Ok(host_port) => {
                         let mut locked_queue = shared_queue.lock().await;
                         let item = QueueItem {
                              name: container_name.clone(),
                              external_port: host_port.to_string(),
                         };
                         locked_queue.push_back(item);
                         println!("Successfully created container '{}' on port {}", container_name, host_port);
                    }
                    Err(e) => eprintln!("Failed to create container '{}': {:?}", container_name, e),
               }
          }
     }

     Ok(shared_queue)
}
