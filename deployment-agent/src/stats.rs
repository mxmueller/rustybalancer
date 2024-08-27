use bollard::Docker;
use bollard::container::{ListContainersOptions, StatsOptions};
use bollard::errors::Error;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time;
use futures::future::join_all;

const WC: f64 = 0.5;  // Weight for CPU
const WM: f64 = 0.5;  // Weight for Memory

#[derive(Serialize, Clone, Debug)]
pub struct ContainerStatus {
    pub id: String,
    pub name: String,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub score: f64,
    pub utilization_category: String,
}

pub async fn get_container_statuses() -> Result<Vec<ContainerStatus>, Error> {
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

    dotenv::dotenv().ok();
    let app_identifier = env::var("APP_IDENTIFIER").expect("APP_IDENTIFIER must be set");

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

    let futures = containers.into_iter().map(|container| {
        let docker = docker.clone();
        async move {
            if let Some(id) = &container.id {
                let name = container.names.unwrap_or_default().get(0).cloned().unwrap_or_default();
                let status = get_single_container_status(&docker, id, name.clone()).await;
                status
            } else {
                Err(Error::IOError { err: std::io::Error::new(std::io::ErrorKind::Other, "No container ID") })
            }
        }
    });

    let results = join_all(futures).await;
    let statuses: Vec<ContainerStatus> = results.into_iter().filter_map(Result::ok).collect();

    Ok(statuses)
}

fn calculate_normalized_cpu_usage(cpu_usage_percent: f64, num_cpus: f64) -> f64 {
    (cpu_usage_percent / num_cpus).min(100.0)
}

fn calculate_score(cpu_usage_percent: f64, memory_usage_percent: f64, num_cpus: f64) -> f64 {
    let normalized_cpu_usage = calculate_normalized_cpu_usage(cpu_usage_percent, num_cpus);
    WC * normalized_cpu_usage + WM * memory_usage_percent
}

fn calculate_memory_usage(stats: &bollard::container::Stats) -> f64 {
    stats.memory_stats.usage.unwrap_or(0) as f64 / stats.memory_stats.limit.unwrap_or(1) as f64 * 100.0
}

fn categorize_utilization(score: f64) -> String {
    match score {
        s if s < 33.0 => "LU".to_string(),   // Low Utilization
        s if s < 66.0 => "MU".to_string(),   // Medium Utilization
        _ => "HU".to_string(),               // High Utilization
    }
}

async fn get_single_container_status(docker: &Docker, container_id: &str, container_name: String) -> Result<ContainerStatus, Error> {
    let mut stats_stream = docker.stats(container_id, Some(StatsOptions{
        stream: true,
        one_shot: false,
    }));

    let stats1 = stats_stream.next().await.unwrap()?;
    time::sleep(Duration::from_millis(100)).await;
    let stats2 = stats_stream.next().await.unwrap()?;

    let cpu_delta = stats2.cpu_stats.cpu_usage.total_usage - stats1.cpu_stats.cpu_usage.total_usage;
    let system_delta = stats2.cpu_stats.system_cpu_usage.unwrap_or(0) - stats1.cpu_stats.system_cpu_usage.unwrap_or(0);
    let num_cpus = stats2.cpu_stats.online_cpus.unwrap_or(1) as f64;

    let cpu_usage_percent = if system_delta > 0 && cpu_delta > 0 {
        (cpu_delta as f64 / system_delta as f64) * num_cpus * 100.0
    } else {
        0.0
    };

    let memory_usage_percent = calculate_memory_usage(&stats2);
    let score = calculate_score(cpu_usage_percent, memory_usage_percent, num_cpus);
    let utilization_category = categorize_utilization(score);

    Ok(ContainerStatus {
        id: container_id.to_string(),
        name: container_name,
        cpu_usage_percent,
        memory_usage_percent,
        score,
        utilization_category,
    })
}