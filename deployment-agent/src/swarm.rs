use bollard::Docker;
use bollard::service::{TaskSpec, ServiceSpec, EndpointSpec, TaskSpecContainerSpec, EndpointPortConfig};
use bollard::models::{ServiceCreateResponse, EndpointSpecModeEnum, EndpointPortConfigProtocolEnum, EndpointPortConfigPublishModeEnum};
use bollard::errors::Error;
use dotenv::dotenv;
use std::env;
use rand::Rng;

pub async fn swarm_create(service_name: &str, image_name: &str, target_port: u16) -> Result<u16, Error> {
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    // Generiere einen zufälligen Host-Port zwischen 10000 und 60000
    let host_port: u16 = rand::thread_rng().gen_range(10000..60000);

    let container_spec = TaskSpecContainerSpec {
        image: Some(image_name.to_string()),
        ..Default::default()
    };

    let task_spec = TaskSpec {
        container_spec: Some(container_spec),
        ..Default::default()
    };

    let port_config = EndpointPortConfig {
        name: None,
        protocol: Some(EndpointPortConfigProtocolEnum::TCP),
        target_port: Some(target_port.into()), // Konvertiere u16 zu i64
        published_port: Some(host_port.into()), // Konvertiere u16 zu i64
        publish_mode: Some(EndpointPortConfigPublishModeEnum::INGRESS),
    };

    let service_spec = ServiceSpec {
        name: Some(service_name.to_string()),
        task_template: Some(task_spec),
        endpoint_spec: Some(EndpointSpec {
            mode: Some(EndpointSpecModeEnum::VIP),
            ports: Some(vec![port_config]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let create_response: ServiceCreateResponse = docker
        .create_service(service_spec, None)
        .await?;

    println!("Created service '{}' with ID: {:?}", service_name, create_response.id);
    println!("Service '{}' is available on port: {}", service_name, host_port);

    Ok(host_port)
}

pub async fn swarm_boot() -> Result<(), Error> {
    dotenv().ok();
    let image_name = env::var("DOCKER_IMAGE").expect("DOCKER_IMAGE must be set");
    let target_port: u16 = env::var("TARGET_PORT")
        .expect("TARGET_PORT must be set")
        .parse()
        .expect("TARGET_PORT must be a valid number");

    let default_container: i16 = env::var("DEFAULT_CONTAINER")
        .expect("TARGET_PORT must be set")
        .parse()
        .expect("TARGET_PORT must be a valid number");

    for i in 1..=default_container {
        let service_name = format!("worker-{}", i);
        swarm_create(&service_name, &image_name, target_port).await?;
    }

    Ok(())
}
