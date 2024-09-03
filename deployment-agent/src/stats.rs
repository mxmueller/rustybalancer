use bollard::Docker;
use bollard::container::{ListContainersOptions, StatsOptions};
use bollard::errors::Error;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::env;
use futures::stream::StreamExt;
use std::time::{Duration, Instant};
use tokio::time;
use std::sync::Arc;
use tokio::sync::Mutex;
use lazy_static::lazy_static;
use tokio::net::TcpStream;
use tokio::time::timeout;

const WC: f64 = 0.35;  // Weight for CPU
const WM: f64 = 0.25; // Weight for Memory
const WN: f64 = 0.15; // Weight for Network
const WA: f64 = 0.25;  // Weight for Availability

const HISTORY_SIZE: usize = 20;
const BEST_TIME_WINDOW: usize = 10;
const EMA_ALPHA: f64 = 0.2; // Exponential Moving Average smoothing factor

#[derive(Serialize, Clone, Debug)]
pub struct ContainerStatus {
    pub id: String,
    pub name: String,
    pub cpu_score: f64,
    pub memory_score: f64,
    pub network_score: f64,
    pub availability_score: f64,
    pub overall_score: f64,
    pub utilization_category: String,
}

#[derive(Clone, Debug)]
struct ResponseTimeMetrics {
    history: VecDeque<f64>,
    best_times: VecDeque<f64>,
    ema_score: f64,
    dynamic_threshold: f64,
}

impl ResponseTimeMetrics {
    fn new() -> Self {
        ResponseTimeMetrics {
            history: VecDeque::with_capacity(HISTORY_SIZE),
            best_times: VecDeque::with_capacity(BEST_TIME_WINDOW),
            ema_score: 100.0,
            dynamic_threshold: 1.0,
        }
    }

    fn add_measurement(&mut self, response_time: f64) {
        if self.history.len() >= HISTORY_SIZE {
            self.history.pop_front();
        }
        self.history.push_back(response_time);

        if self.best_times.is_empty() || response_time < *self.best_times.back().unwrap() {
            if self.best_times.len() >= BEST_TIME_WINDOW {
                self.best_times.pop_front();
            }
            self.best_times.push_back(response_time);
        }

        let avg_time = self.calculate_average();
        self.dynamic_threshold = (self.dynamic_threshold * 0.9 + avg_time * 1.5 * 0.1).max(0.5);
    }

    fn get_best_time(&self) -> f64 {
        self.best_times.iter().sum::<f64>() / self.best_times.len() as f64
    }

    fn calculate_average(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history.iter().sum::<f64>() / self.history.len() as f64
    }

    fn calculate_trend(&self) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }
        let recent = self.history.iter().rev().take(5).sum::<f64>() / 5.0;
        let older = self.history.iter().rev().skip(5).take(5).sum::<f64>() / 5.0;
        (older - recent) / older
    }

    fn calculate_availability_score(&mut self) -> f64 {
        if self.history.is_empty() {
            return 100.0;
        }

        let current_time = *self.history.back().unwrap();
        let avg_time = self.calculate_average();
        let best_time = self.get_best_time();
        let trend = self.calculate_trend();

        let effective_time = 0.3 * current_time + 0.7 * avg_time;
        let ratio = effective_time / best_time;
        let base_score = 100.0 * (1.0 / ratio).powf(1.5);

        let penalty = if effective_time > self.dynamic_threshold {
            let over_threshold = effective_time - self.dynamic_threshold;
            20.0 * (1.0 - (-over_threshold).exp())
        } else {
            0.0
        };

        let trend_adjustment = trend * 10.0;

        let raw_score = (base_score - penalty + trend_adjustment).max(0.0).min(100.0);

        self.ema_score = EMA_ALPHA * raw_score + (1.0 - EMA_ALPHA) * self.ema_score;

        self.ema_score
    }
}

lazy_static! {
    static ref NETWORK_USAGE: Arc<Mutex<HashMap<String, f64>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref RESPONSE_TIME_METRICS: Arc<Mutex<HashMap<String, ResponseTimeMetrics>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub async fn get_container_statuses() -> Result<Vec<ContainerStatus>, Error> {
    let docker = Docker::connect_with_unix_defaults().expect("Failed to connect to Docker");

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

    let futures: Vec<_> = containers.into_iter().filter_map(|container| {
        container.id.map(|id| {
            let docker = docker.clone();
            let name = container.names.unwrap_or_default().get(0).cloned().unwrap_or_default();
            tokio::spawn(async move {
                get_single_container_status(&docker, &id, name).await
            })
        })
    }).collect();

    let results = futures::future::join_all(futures).await;
    let statuses: Vec<ContainerStatus> = results.into_iter()
        .filter_map(|r| r.ok().and_then(|inner_result| inner_result.ok()))
        .collect();

    Ok(statuses)
}

async fn get_single_container_status(docker: &Docker, container_id: &str, container_name: String) -> Result<ContainerStatus, Error> {
    let mut stats_stream = docker.stats(container_id, Some(StatsOptions{
        stream: true,
        one_shot: false,
    }));

    let stats1 = stats_stream.next().await.unwrap()?;
    let start_time = Instant::now();
    time::sleep(Duration::from_secs(1)).await;
    let stats2 = stats_stream.next().await.unwrap()?;
    let duration = start_time.elapsed();

    let cpu_usage_percent = calculate_cpu_usage(&stats1, &stats2);
    let memory_usage_percent = calculate_memory_usage(&stats2);
    let network_usage_percent = calculate_network_usage(container_id, &stats1, &stats2, duration).await;

    let cpu_score = 100.0 - cpu_usage_percent;
    let memory_score = 100.0 - memory_usage_percent;
    let network_score = 100.0 - network_usage_percent;

    let availability_score = check_container_availability(docker, container_id).await;

    let overall_score = calculate_score(cpu_score, memory_score, network_score, availability_score);
    let utilization_category = categorize_utilization(overall_score);

    Ok(ContainerStatus {
        id: container_id.to_string(),
        name: container_name,
        cpu_score,
        memory_score,
        network_score,
        availability_score,
        overall_score,
        utilization_category,
    })
}

fn calculate_cpu_usage(stats1: &bollard::container::Stats, stats2: &bollard::container::Stats) -> f64 {
    let cpu_delta = stats2.cpu_stats.cpu_usage.total_usage - stats1.cpu_stats.cpu_usage.total_usage;
    let system_delta = stats2.cpu_stats.system_cpu_usage.unwrap_or(0) - stats1.cpu_stats.system_cpu_usage.unwrap_or(0);
    let num_cpus = stats2.cpu_stats.online_cpus.unwrap_or(1) as f64;

    if system_delta > 0 && cpu_delta > 0 {
        (cpu_delta as f64 / system_delta as f64) * num_cpus * 100.0
    } else {
        0.0
    }
}

fn calculate_memory_usage(stats: &bollard::container::Stats) -> f64 {
    let usage = stats.memory_stats.usage.unwrap_or(0) as f64;
    let limit = stats.memory_stats.limit.unwrap_or(1) as f64;
    (usage / limit) * 100.0
}

async fn calculate_network_usage(container_id: &str, stats1: &bollard::container::Stats, stats2: &bollard::container::Stats, duration: Duration) -> f64 {
    let duration_secs = duration.as_secs_f64();

    let rx_bytes1: u64 = stats1.networks.as_ref().map(|n| n.values().map(|i| i.rx_bytes).sum()).unwrap_or(0);
    let tx_bytes1: u64 = stats1.networks.as_ref().map(|n| n.values().map(|i| i.tx_bytes).sum()).unwrap_or(0);
    let rx_bytes2: u64 = stats2.networks.as_ref().map(|n| n.values().map(|i| i.rx_bytes).sum()).unwrap_or(0);
    let tx_bytes2: u64 = stats2.networks.as_ref().map(|n| n.values().map(|i| i.tx_bytes).sum()).unwrap_or(0);

    let total_bytes = (rx_bytes2.saturating_sub(rx_bytes1) + tx_bytes2.saturating_sub(tx_bytes1)) as f64;
    let mb_per_second = total_bytes / duration_secs / 1_000_000.0;

    let mut network_usage = NETWORK_USAGE.lock().await;
    let prev_usage = network_usage.entry(container_id.to_string()).or_insert(0.0);
    let usage_percent = if *prev_usage > 0.0 {
        ((mb_per_second - *prev_usage) / *prev_usage * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    *prev_usage = mb_per_second;

    usage_percent
}

fn calculate_score(cpu_score: f64, memory_score: f64, network_score: f64, availability_score: f64) -> f64 {
    let score = WC * cpu_score +
        WM * memory_score +
        WN * network_score +
        WA * availability_score;
    score.min(100.0).max(0.0)
}

fn categorize_utilization(score: f64) -> String {
    match score {
        s if s >= 70.0 => "LU",   // Low Utilization (Good performance)
        s if s >= 40.0 => "MU",   // Medium Utilization
        _ => "HU",                // High Utilization (Poor performance)
    }.to_string()
}

async fn check_container_availability(docker: &Docker, container_id: &str) -> f64 {
    let response_time = get_container_response_time(docker, container_id).await;
    calculate_availability_score(container_id, response_time).await
}

async fn get_container_response_time(docker: &Docker, container_id: &str) -> Option<f64> {
    match docker.inspect_container(container_id, None).await {
        Ok(info) => {
            let container_name = info.name.unwrap_or_default().trim_start_matches('/').to_string();

            if let Some(network_settings) = &info.network_settings {
                if let Some(networks) = &network_settings.networks {
                    if let Some(network) = networks.get("rust-network") {
                        if let Some(ip_address) = &network.ip_address {
                            if let Some(ports) = &network_settings.ports {
                                for (container_port, _) in ports {
                                    let port = container_port.split('/').next().unwrap_or("5000");
                                    let addr = format!("{}:{}", ip_address, port);

                                    let start = Instant::now();
                                    match timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await {
                                        Ok(Ok(_)) => {
                                            let duration = start.elapsed().as_secs_f64();
                                            return Some(duration);
                                        },
                                        Ok(Err(_)) | Err(_) => continue,
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None
        },
        Err(_) => None,
    }
}

async fn calculate_availability_score(container_id: &str, response_time: Option<f64>) -> f64 {
    let mut metrics_map = RESPONSE_TIME_METRICS.lock().await;
    let metrics = metrics_map.entry(container_id.to_string()).or_insert_with(ResponseTimeMetrics::new);

    match response_time {
        Some(time) => {
            metrics.add_measurement(time);
            metrics.calculate_availability_score()
        },
        None => 0.0 // Worst score for errors or missing data
    }
}