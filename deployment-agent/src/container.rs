use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, ListContainersOptions, StartContainerOptions};
use bollard::models::{HostConfig, PortBinding};
use bollard::errors::Error;
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use rand::Rng;
use bollard::image::CreateImageOptions;
use futures_util::stream::StreamExt;

async fn pull_image(docker: &Docker, image_name: &str) -> Result<(), Error> {
    let create_image_options = CreateImageOptions {
        from_image: image_name,
        ..Default::default()
    };

    let mut stream = docker.create_image(Some(create_image_options), None, None);

    while let Some(pull_result) = stream.next().await {
        match pull_result {
            Ok(output) => println!("{:?}", output),
            Err(e) => eprintln!("Error while pulling image: {:?}", e),
        }
    }

    Ok(())
}

pub async fn create_container(container_name: &str, image_name: &str, target_port: u16, app_identifier: &str) -> Result<u16, Error> {
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    // Pull the image if it's not already available locally
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
        platform: None,
    };

    let create_response = docker
        .create_container(Some(create_options), config)
        .await?;

    println!("Created container '{}' with ID: {:?}", container_name, create_response.id);

    docker.start_container(&create_response.id, None::<StartContainerOptions<String>>).await?;

    println!("Container '{}' is available on port: {}", container_name, host_port);

    Ok(host_port)
}

pub async fn start_containers() -> Result<(), Error> {
    dotenv().ok();
    let image_name = env::var("DOCKER_IMAGE").expect("DOCKER_IMAGE must be set");
    let target_port: u16 = env::var("TARGET_PORT")
        .expect("TARGET_PORT must be set")
        .parse()
        .expect("TARGET_PORT must be a valid number");

    let default_container: i16 = env::var("DEFAULT_CONTAINER")
        .expect("DEFAULT_CONTAINER must be set")
        .parse()
        .expect("DEFAULT_CONTAINER must be a valid number");

    let app_identifier = env::var("APP_IDENTIFIER").expect("APP_IDENTIFIER must be set");

    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    // Check the number of existing containers with the specific label
    let filters = {
        let mut map = HashMap::new();
        map.insert("label".to_string(), vec![format!("application={}", app_identifier)]);
        map
    };

    let options = ListContainersOptions {
        all: true,
        filters,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;
    let existing_container_count = containers.len() as i16;

    if existing_container_count >= default_container {
        println!("There are already {} containers running. No additional containers needed.", existing_container_count);
        return Ok(());
    }

    for i in existing_container_count + 1..=default_container {
        let container_name = format!("worker-{}", i);
        match create_container(&container_name, &image_name, target_port, &app_identifier).await {
            Ok(host_port) => println!("Successfully created container '{}' on port {}", container_name, host_port),
            Err(e) => eprintln!("Failed to create container '{}': {:?}", container_name, e),
        }
    }

    Ok(())
}
