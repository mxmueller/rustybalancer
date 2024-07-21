use bollard::Docker;
use bollard::container::{InspectContainerOptions, ListContainersOptions, StatsOptions};
use bollard::errors::Error;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use futures::stream::StreamExt;
use chrono::DateTime;
use chrono::Utc;

#[derive(Serialize)]
pub struct ContainerStatus {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub ports: HashMap<String, Vec<String>>,
    pub cpu_usage: f64,
    pub memory_usage: f64, // Hier bleibt es f64
    pub uptime: String,
}

pub async fn get_container_status() -> Result<Vec<ContainerStatus>, Error> {
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    // Laden der Umgebungsvariablen
    dotenv::dotenv().ok();
    let app_identifier = env::var("APP_IDENTIFIER").expect("APP_IDENTIFIER must be set");

    // Filter setzen, um nur Container mit dem spezifischen Label zu erhalten
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

    println!("Found {} containers", containers.len());

    let mut status_list = Vec::new();

    for container in containers {
        println!("Inspecting container ID: {:?}", container.id);

        let details = docker
            .inspect_container(container.id.as_deref().unwrap_or(""), None::<InspectContainerOptions>)
            .await?;

        let mut stats_stream = docker
            .stats(container.id.as_deref().unwrap_or(""), Some(StatsOptions { stream: false, one_shot: true }))
            .take(1);

        let stats = stats_stream.next().await.unwrap()?;

        let cpu_usage = calculate_cpu_usage(&stats);
        let memory_usage = calculate_memory_usage(&stats);

        let uptime = details
            .state
            .as_ref()
            .and_then(|state| state.started_at.as_ref())
            .map(|start| format_duration(Utc::now().signed_duration_since(DateTime::parse_from_rfc3339(start).unwrap())))
            .unwrap_or_default();

        let ports = details
            .network_settings
            .map(|ns| {
                ns.ports.map(|ps| {
                    ps.iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                v.clone().unwrap_or(vec![])
                                    .iter()
                                    .map(|pb| format!("{}:{}", pb.host_ip.clone().unwrap_or_default(), pb.host_port.clone().unwrap_or_default()))
                                    .collect(),
                            )
                        })
                        .collect()
                })
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let state = details
            .state
            .as_ref()
            .and_then(|state| state.status.as_ref())
            .map(|status| status.to_string())
            .unwrap_or_default();

        status_list.push(ContainerStatus {
            id: details.id.unwrap_or_default(),
            name: details.name.unwrap_or_default(),
            image: details.config.and_then(|config| config.image).unwrap_or_default(),
            state,
            ports,
            cpu_usage,
            memory_usage,
            uptime,
        });
    }

    Ok(status_list)
}

fn calculate_cpu_usage(stats: &bollard::container::Stats) -> f64 {
    let cpu_delta = stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage;
    let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) - stats.precpu_stats.system_cpu_usage.unwrap_or(0);
    if system_delta > 0 && cpu_delta > 0 {
        (cpu_delta as f64 / system_delta as f64) * stats.cpu_stats.online_cpus.unwrap_or(1) as f64 * 100.0
    } else {
        0.0
    }
}

fn calculate_memory_usage(stats: &bollard::container::Stats) -> f64 {
    if let Some(usage) = stats.memory_stats.usage {
        if let Some(limit) = stats.memory_stats.limit {
            return (usage as f64 / limit as f64) * 100.0;
        }
    }
    0.0
}

fn format_duration(duration: chrono::Duration) -> String {
    let seconds = duration.num_seconds();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{}h {}m {}s", hours, minutes, seconds)
}
