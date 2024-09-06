use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, ListContainersOptions, StartContainerOptions, StopContainerOptions, RemoveContainerOptions};
use bollard::models::{HostConfig, PortBinding, ContainerSummary as APIContainers};
use bollard::errors::Error;
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use rand::Rng;
use bollard::image::CreateImageOptions;
use futures_util::StreamExt;
use md5;
use redis::Commands;
use uuid::Uuid;
use crate::db;
use crate::queue::QueueItem;
use std::time::Duration;
use tokio::time::sleep;

pub fn generate_hash_based_key(app_identifier: &str, container_name: &str) -> String {
    let data = format!("{}:{}", app_identifier, container_name);
    let digest = md5::compute(data);
    format!("container:{:x}", digest)
}

fn store_container_info_init(
    conn: &mut redis::Connection,
    key: &str,
    port: u16,
    image: &str,
) -> redis::RedisResult<()> {
    println!("Storing container info for key: {}", key);
    let _: () = conn.hset(key, "category", "INIT")?;
    let _: () = conn.hset(key, "score", "100")?;
    let _: () = conn.hset(key, "port", port.to_string())?;
    let _: () = conn.hset(key, "image", image)?;

    // Verify that all fields were set correctly
    let fields: HashMap<String, String> = conn.hgetall(key)?;
    if fields.len() != 4 || !fields.contains_key("category") || !fields.contains_key("score")
        || !fields.contains_key("port") || !fields.contains_key("image") {
        println!("Not all fields were set correctly for key: {}. Fields: {:?}", key, fields);
    } else {
        println!("Container info stored successfully for key: {}", key);
    }

    Ok(())
}

pub fn update_container_category(
    conn: &mut redis::Connection,
    key: &str,
    category: &str,
) -> redis::RedisResult<()> {
    println!("Updating container category for key: {} to {}", key, category);
    conn.hset(key, "category", category)
}

async fn pull_image(docker: &Docker, image_name: &str) -> Result<(), Error> {
    println!("Pulling image: {}", image_name);
    let create_image_options = CreateImageOptions {
        from_image: image_name,
        ..Default::default()
    };
    let mut retry_count = 0;
    let max_retries = 3;
    let mut delay = Duration::from_secs(5);

    while retry_count < max_retries {
        let mut stream = docker.create_image(Some(create_image_options.clone()), None, None);
        let mut success = true;

        while let Some(pull_result) = stream.next().await {
            match pull_result {
                Ok(output) => println!("Image pull progress: {:?}", output),
                Err(e) => {
                    println!("Error while pulling image: {:?}", e);
                    success = false;
                    break;
                }
            }
        }

        if success {
            println!("Image pulled successfully: {}", image_name);
            return Ok(());
        }

        retry_count += 1;
        if retry_count < max_retries {
            println!("Retrying image pull in {:?}...", delay);
            sleep(delay).await;
            delay *= 2; // Exponential backoff
        }
    }

    println!("Max retries reached while pulling image: {}", image_name);
    Err(Error::IOError { err: std::io::Error::new(std::io::ErrorKind::Other, "Max retries reached while pulling image") })
}

pub async fn create_container(
    container_name: &str,
    image_name: &str,
    target_port: u16,
    app_identifier: &str,
    conn: &mut redis::Connection,
) -> Result<String, Error> {
    println!("Creating container: {}", container_name);
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    pull_image(&docker, image_name).await?;

    let host_port: u16 = rand::thread_rng().gen_range(30000..40000);

    let port_bindings = {
        let mut map = HashMap::new();
        map.insert(
            format!("{}/tcp", target_port),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(host_port.to_string()),
            }]),
        );
        map
    };

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        network_mode: Some("rust-network".to_string()),
        ..Default::default()
    };

    let mut labels = HashMap::new();
    labels.insert("application".to_string(), app_identifier.to_string());

    let config = Config {
        image: Some(image_name.to_string()),
        host_config: Some(host_config),
        labels: Some(labels),
        ..Default::default()
    };

    let create_options = CreateContainerOptions {
        name: container_name,
        ..Default::default()
    };

    let key = generate_hash_based_key(app_identifier, container_name);

    let mut retry_count = 0;
    let max_retries = 3;
    let mut delay = Duration::from_secs(5);

    while retry_count < max_retries {
        match docker.create_container(Some(create_options.clone()), config.clone()).await {
            Ok(create_response) => {
                println!("Container created successfully: {}", container_name);

                // Store container info in Redis
                if let Err(e) = store_container_info_init(conn, &key, host_port, image_name) {
                    println!("Failed to store container info in Redis: {:?}", e);
                    // Consider whether to proceed or return an error here
                }

                // Start the container
                match docker.start_container(&create_response.id, None::<StartContainerOptions<String>>).await {
                    Ok(_) => {
                        println!("Container started successfully: {}", container_name);
                        return Ok(container_name.to_string());
                    },
                    Err(e) => {
                        println!("Failed to start container: {:?}", e);
                        // Consider cleanup actions here
                        return Err(e);
                    }
                }
            },
            Err(e) => {
                println!("Error creating container {} (attempt {}): {:?}", container_name, retry_count + 1, e);
                retry_count += 1;
                if retry_count < max_retries {
                    println!("Retrying creation of container {} in {:?}...", container_name, delay);
                    sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }
    }

    println!("Max retries reached while creating container: {}", container_name);
    Err(Error::IOError { err: std::io::Error::new(std::io::ErrorKind::Other, "Max retries reached while creating container") })
}

pub async fn create_single_container(
    image_name: &str,
    target_port: u16,
    app_identifier: &str,
    conn: &mut redis::Connection,
) -> Result<QueueItem, Error> {
    let uuid = Uuid::new_v4();
    let container_name = format!("worker-{}", &uuid.to_string()[..8]);

    match create_container(&container_name, image_name, target_port, app_identifier, conn).await {
        Ok(dns_name) => {
            let item = QueueItem {
                dns_name,
                score: 100.0,
                utilization_category: "LU".to_string(),
            };
            Ok(item)
        }
        Err(e) => Err(e)
    }
}

pub async fn list_running_containers(app_identifier: &str) -> Result<Vec<APIContainers>, Error> {
    println!("Listing running containers for app: {}", app_identifier);
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    let filters = {
        let mut map = HashMap::new();
        map.insert("label".to_string(), vec![format!("application={}", app_identifier)]);
        map.insert("status".to_string(), vec!["running".to_string()]);
        map
    };

    let options = ListContainersOptions {
        all: false,
        filters,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;
    println!("Found {} running containers for app: {}", containers.len(), app_identifier);
    Ok(containers)
}

pub async fn check_and_stop_container_if_not_in_db(
    container: &APIContainers,
    conn: &mut redis::Connection,
    app_identifier: &str,
) -> Result<Option<QueueItem>, Error> {
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    let container_name = container.names.as_ref()
        .and_then(|names| names.get(0).cloned())
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string();

    println!("Checking container: {}", container_name);

    let key = generate_hash_based_key(app_identifier, &container_name);

    if db::check_config_value_exists(conn, &key) {
        let category: String = conn.hget(&key, "category").unwrap_or_else(|_| "Unknown".to_string());
        println!("Container {} found in DB with category: {}", container_name, category);
        Ok(Some(QueueItem {
            dns_name: container_name,
            score: 100.0,
            utilization_category: category,
        }))
    } else {
        println!("Container {} not found in DB, stopping and removing", container_name);
        docker.stop_container(container.id.as_deref().unwrap_or(""), None::<StopContainerOptions>).await?;
        docker.remove_container(container.id.as_deref().unwrap_or(""), None::<RemoveContainerOptions>).await?;
        Ok(None)
    }
}

pub fn cleanup_orphaned_db_entries(
    conn: &mut redis::Connection,
    running_container_keys: &HashMap<String, String>,
) -> Result<(), Error> {
    println!("Cleaning up orphaned DB entries");
    let keys: Vec<String> = conn.keys("container:*")
        .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    for key in keys {
        if !running_container_keys.contains_key(&key) {
            println!("Deleting orphaned database entry '{}'", key);
            let _: () = conn.del(&key)
                .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        }
    }

    Ok(())
}

pub async fn manage_containers(app_identifier: &str, default_container: i16) -> Result<Vec<QueueItem>, Error> {
    println!("Managing containers for app: {}", app_identifier);
    let mut conn = db::get_redis_connection();
    let image_name = env::var("DOCKER_IMAGE").expect("DOCKER_IMAGE must be set");
    let target_port: u16 = env::var("TARGET_PORT")
        .expect("TARGET_PORT must be set")
        .parse()
        .expect("TARGET_PORT must be a valid number");

    let containers = list_running_containers(app_identifier).await?;

    let mut running_containers = HashMap::new();
    let mut queue_items = Vec::new();

    // Step 1: Process existing containers
    for container in containers {
        let container_name = container.names.as_ref()
            .and_then(|names| names.get(0).cloned())
            .unwrap_or_default()
            .trim_start_matches('/')
            .to_string();

        let key = generate_hash_based_key(app_identifier, &container_name);

        // Check the database for SUNDOWN status
        let db_category: String = conn.hget(&key, "category").unwrap_or_else(|_| "Unknown".to_string());

        if db_category == "SUNDOWN" {
            // If the container is marked as SUNDOWN in the database, maintain this state
            queue_items.push(QueueItem {
                dns_name: container_name.clone(),
                score: 0.0, // Set a low score for SUNDOWN containers
                utilization_category: "SUNDOWN".to_string(),
            });
        } else {
            // For non-SUNDOWN containers, proceed with normal processing
            if let Some(item) = check_and_stop_container_if_not_in_db(&container, &mut conn, app_identifier).await? {
                if let Some(id) = &container.id {
                    running_containers.insert(generate_hash_based_key(app_identifier, &item.dns_name), id.clone());
                }
                queue_items.push(item);
            }
        }
    }

    cleanup_orphaned_db_entries(&mut conn, &running_containers)?;

    let running_containers_count = running_containers.len() as i16;
    println!("Current running containers: {}, Default containers: {}", running_containers_count, default_container);

    // Step 2: Create new containers if needed
    if running_containers_count < default_container {
        for i in running_containers_count + 1..=default_container {
            println!("Creating container {} of {}", i, default_container);
            match create_single_container(&image_name, target_port, app_identifier, &mut conn).await {
                Ok(item) => {
                    println!("Container created successfully: {}", item.dns_name);
                    queue_items.push(item);
                }
                Err(e) => println!("Failed to create container: {:?}", e),
            }
        }
    }

    Ok(queue_items)
}
pub async fn remove_container(app_identifier: &str, container_name: &str) -> Result<(), Error> {
    println!("Removing container: {}", container_name);
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    // First, stop the container
    match docker.stop_container(container_name, None::<StopContainerOptions>).await {
        Ok(_) => println!("Container stopped: {}", container_name),
        Err(e) => println!("Error stopping container {}: {:?}", container_name, e),
    }

    // Then, remove the container
    match docker.remove_container(
        container_name,
        Some(RemoveContainerOptions {
            force: true,
            ..Default::default()
        }),
    ).await {
        Ok(_) => println!("Container removed: {}", container_name),
        Err(e) => println!("Error removing container {}: {:?}", container_name, e),
    }

    // Remove container info from Redis
    let mut conn = db::get_redis_connection();
    let key = generate_hash_based_key(app_identifier, container_name);
    match conn.del::<_, ()>(&key) {
        Ok(_) => println!("Container info removed from Redis: {}", container_name),
        Err(e) => println!("Error removing container info from Redis: {:?}", e),
    }

    Ok(())
}