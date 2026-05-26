use std::time::Duration;

use chrono::Utc;
use sqlx::SqlitePool;
use tokio::time;
use tracing::{error, info, warn};

use crate::db::sqlite;
use crate::models::{RemoteHealthResponse, RemoteServiceStatus, ServiceStatus};

/// Periodic poller that checks each registered service's health endpoint.
pub struct Poller {
    pool: SqlitePool,
    default_interval_secs: u64,
    client: reqwest::Client,
}

impl Poller {
    pub fn new(pool: SqlitePool, default_interval_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            pool,
            default_interval_secs,
            client,
        }
    }

    /// Run the poller loop. This will poll all services at their configured intervals.
    pub async fn run(self) {
        let mut interval = time::interval(Duration::from_secs(self.default_interval_secs));

        info!(
            "Poller started with default interval of {}s",
            self.default_interval_secs
        );

        loop {
            interval.tick().await;

            let services = match sqlite::list_services(&self.pool).await {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to list services for polling: {}", e);
                    continue;
                }
            };

            if services.is_empty() {
                continue;
            }

            for svc in services {
                let pool = self.pool.clone();
                let client = self.client.clone();
                let base_url = svc.base_url.clone();
                let service_id = svc.id;
                let service_name = svc.name.clone();

                tokio::spawn(async move {
                    if let Err(e) =
                        Self::poll_service(&pool, &client, service_id, &service_name, &base_url)
                            .await
                    {
                        warn!("Poll failed for service '{}' ({}): {}", service_name, service_id, e);
                    }
                });
            }

            // Periodic cleanup: keep last 10000 records per service
            if let Err(e) = sqlite::cleanup_old_statuses(&self.pool, 10000).await {
                warn!("Status cleanup failed: {}", e);
            }
        }
    }

    /// Poll a single service and store the result.
    async fn poll_service(
        pool: &SqlitePool,
        client: &reqwest::Client,
        service_id: i64,
        service_name: &str,
        base_url: &str,
    ) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        // Try /api/status first, fall back to /api/health
        let result = Self::fetch_status(client, base_url).await;
        let response_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        let status = match result {
            Ok((status_data, health_data)) => {
                // Merge status and health data, preferring status
                let status_str = status_data
                    .as_ref()
                    .and_then(|s| s.status.clone())
                    .or_else(|| health_data.as_ref().and_then(|h| h.status.clone()))
                    .unwrap_or_else(|| "healthy".to_string());

                let availability_level = status_data
                    .as_ref()
                    .and_then(|s| s.availability_level.clone())
                    .or_else(|| health_data.as_ref().and_then(|h| h.availability_level.clone()))
                    .unwrap_or_else(|| "available".to_string());

                let models_loaded = status_data
                    .as_ref()
                    .and_then(|s| s.models_loaded)
                    .or_else(|| health_data.as_ref().and_then(|h| h.models_loaded))
                    .unwrap_or(false);

                let pending_requests = status_data
                    .as_ref()
                    .and_then(|s| s.pending_requests)
                    .or_else(|| health_data.as_ref().and_then(|h| h.pending_requests))
                    .unwrap_or(0);

                let queue_capacity = status_data
                    .as_ref()
                    .and_then(|s| s.queue_capacity)
                    .or_else(|| health_data.as_ref().and_then(|h| h.queue_capacity))
                    .unwrap_or(0);

                let utilization_percent = if queue_capacity > 0 {
                    (pending_requests as f64 / queue_capacity as f64) * 100.0
                } else {
                    0.0
                };

                ServiceStatus {
                    id: 0, // Auto-generated
                    service_id,
                    status: status_str,
                    availability_level,
                    models_loaded,
                    pending_requests,
                    queue_capacity,
                    utilization_percent,
                    avg_response_ms: status_data.as_ref().and_then(|s| s.avg_response_ms),
                    total_requests: status_data.as_ref().and_then(|s| s.total_requests),
                    success_count: status_data.as_ref().and_then(|s| s.success_count),
                    failure_count: status_data.as_ref().and_then(|s| s.failure_count),
                    polled_at: Utc::now(),
                    response_time_ms,
                }
            }
            Err(_) => {
                // Service is unreachable
                ServiceStatus {
                    id: 0,
                    service_id,
                    status: "unavailable".to_string(),
                    availability_level: "unavailable".to_string(),
                    models_loaded: false,
                    pending_requests: 0,
                    queue_capacity: 0,
                    utilization_percent: 0.0,
                    avg_response_ms: None,
                    total_requests: None,
                    success_count: None,
                    failure_count: None,
                    polled_at: Utc::now(),
                    response_time_ms,
                }
            }
        };

        tracing::debug!(
            "Polled '{}' ({}): status={}, response_time={:.1}ms",
            service_name,
            service_id,
            status.status,
            status.response_time_ms
        );

        sqlite::insert_status(pool, &status).await?;
        Ok(())
    }

    /// Attempt to fetch /api/status and /api/health from the service.
    async fn fetch_status(
        client: &reqwest::Client,
        base_url: &str,
    ) -> anyhow::Result<(Option<RemoteServiceStatus>, Option<RemoteHealthResponse>)> {
        let url_status = format!("{}/api/status", base_url.trim_end_matches('/'));
        let url_health = format!("{}/api/health", base_url.trim_end_matches('/'));

        // Try /api/status first
        let status_resp = client.get(&url_status).send().await;
        let status_data: Option<RemoteServiceStatus> = match status_resp {
            Ok(resp) if resp.status().is_success() => resp.json().await.ok(),
            _ => None,
        };

        // Also try /api/health
        let health_resp = client.get(&url_health).send().await;
        let health_data: Option<RemoteHealthResponse> = match health_resp {
            Ok(resp) if resp.status().is_success() => resp.json().await.ok(),
            _ => None,
        };

        if status_data.is_none() && health_data.is_none() {
            return Err(anyhow::anyhow!("Both /api/status and /api/health failed"));
        }

        Ok((status_data, health_data))
    }
}
