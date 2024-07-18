use bollard::Docker;
use bollard::container::{StatsOptions, Stats, ListContainersOptions};
use bollard::models::{ContainerInspectResponse, ContainerStateStatusEnum};
use bollard::service::{InspectServiceOptions, Service};
use bollard::errors::Error;
use futures_util::stream::StreamExt;
use serde_json::json;

async fn container_stats(container_id: &str) -> Result<Stats, Error> {
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    let stats_options = Some(StatsOptions {
        stream: false, // Set to false to get a single stats response
        one_shot: true, // Ensure this is set
    });

    let mut stats_stream = docker.stats(container_id, stats_options);
    if let Some(stats_result) = stats_stream.next().await {
        match stats_result {
            Ok(stats) => Ok(stats),
            Err(e) => Err(e),
        }
    } else {
        Err(Error::DockerResponseServerError {
            status_code: 500,
            message: "No stats received".to_string(),
        })
    }
}

async fn container_inspect(container_id: &str) -> Result<ContainerInspectResponse, Error> {
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");
    docker.inspect_container(container_id, None).await
}

pub async fn service_inspect(service_id: &str) -> Result<Service, Error> {
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");
    docker.inspect_service(service_id, None::<InspectServiceOptions>).await
}

fn extract_published_ports(service: &Service) -> String {
    if let Some(endpoint) = &service.endpoint {
        if let Some(ports) = &endpoint.ports {
            return ports
                .iter()
                .map(|port| port.published_port.unwrap_or(0).to_string())
                .collect::<Vec<String>>()
                .join(", ");
        }
    }
    "N/A".to_string()
}

pub async fn display_stats() -> Result<(), Error> {
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    let options = Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    });

    let containers = docker.list_containers(options).await?;
    println!("Found {} containers", containers.len()); // Debug-Ausgabe

    for container in containers {
        if let Some(container_id) = container.id {
            println!("\nFetching stats for container: {}", container_id); // Debug-Ausgabe

            let inspect = container_inspect(&container_id).await?;
            let stats = container_stats(&container_id).await?;

            let name = inspect.name.unwrap_or_else(|| "N/A".to_string());
            let state = inspect.state.unwrap_or_default();
            let status = state.status.map_or("unknown".to_string(), |s| s.to_string());
            let network_settings = inspect.network_settings.unwrap_or_default();
            let ip_address = network_settings.networks.map_or("N/A".to_string(), |n| {
                n.iter().next().map_or("N/A".to_string(), |(_, v)| v.ip_address.clone().unwrap_or_else(|| "N/A".to_string()))
            });

            let cpu_usage = stats.cpu_stats.cpu_usage.total_usage as f64 / stats.cpu_stats.system_cpu_usage.unwrap_or(1) as f64 * 100.0;
            let memory_usage = stats.memory_stats.usage.unwrap_or(0) as f64 / stats.memory_stats.limit.unwrap_or(1) as f64 * 100.0;

            let rx_bytes = stats.networks.as_ref().map_or(0, |networks| {
                networks.values().map(|v| v.rx_bytes).sum::<u64>()
            });
            let tx_bytes = stats.networks.as_ref().map_or(0, |networks| {
                networks.values().map(|v| v.tx_bytes).sum::<u64>()
            });

            println!("Container ID: {}", container_id);
            println!("Name: {}", name);
            println!("Status: {}", status);
            println!("IP Address: {}", ip_address);
            println!("CPU Usage: {:.2}%", cpu_usage);
            println!("Memory Usage: {:.2}%", memory_usage);
            println!("Network Traffic: RX {} bytes, TX {} bytes", rx_bytes, tx_bytes);

            // Extracting service ID from container labels
            if let Some(labels) = inspect.config.and_then(|config| config.labels) {
                if let Some(service_id) = labels.get("com.docker.swarm.service.id") {
                    let service_details = service_inspect(service_id).await?;
                    let published_ports = extract_published_ports(&service_details);
                    println!("Service ID: {}", service_id);
                    println!("Published Ports: {}", published_ports);
                }
            }
        } else {
            println!("Container ID not found"); // Debug-Ausgabe
        }
    }

    Ok(())
}
