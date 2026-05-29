use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A monitored server group (e.g. "Production OCR Cluster").
/// Each server can contain multiple service instances.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MonitorServer {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

/// Request body for creating a new server group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

/// A single service instance belonging to a server group.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServiceInstance {
    pub id: i64,
    pub server_id: i64,
    pub name: String,
    pub service_type: String, // "dotnet-ocr", "cpp-ocr", "rust-ocr"
    pub base_url: String,
    pub poll_interval_secs: i64,
    pub created_at: DateTime<Utc>,
}

/// Request body for registering a new service instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInstanceRequest {
    pub name: String,
    pub service_type: String,
    pub base_url: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: i64,
}

fn default_poll_interval() -> i64 {
    10
}

/// A status snapshot captured during polling.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServiceStatus {
    pub id: i64,
    pub instance_id: i64,
    pub status: String, // "healthy", "busy", "unavailable"
    pub availability_level: String,
    pub models_loaded: bool,
    pub pending_requests: i32,
    pub queue_capacity: i32,
    pub utilization_percent: f64,
    pub avg_response_ms: Option<f64>,
    pub total_requests: Option<i64>,
    pub success_count: Option<i64>,
    pub failure_count: Option<i64>,
    pub polled_at: DateTime<Utc>,
    pub response_time_ms: f64,
}

/// The response we expect from a service's /api/status endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteServiceStatus {
    pub status: Option<String>,
    pub availability_level: Option<String>,
    pub reason: Option<String>,
    pub models_loaded: Option<bool>,
    pub pool_size: Option<i32>,
    pub queue_capacity: Option<i32>,
    pub pending_requests: Option<i32>,
    pub avg_response_ms: Option<f64>,
    pub total_requests: Option<i64>,
    pub success_count: Option<i64>,
    pub failure_count: Option<i64>,
    pub server_name: Option<String>,
}

/// The health response we expect from /api/health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteHealthResponse {
    pub status: Option<String>,
    pub availability_level: Option<String>,
    pub reason: Option<String>,
    pub models_loaded: Option<bool>,
    pub pool_size: Option<i32>,
    pub queue_capacity: Option<i32>,
    pub pending_requests: Option<i32>,
    pub server_name: Option<String>,
}

/// Dashboard summary data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_servers: i64,
    pub total_instances: i64,
    pub healthy_instances: i64,
    pub busy_instances: i64,
    pub unavailable_instances: i64,
    pub servers: Vec<ServerDashboardEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDashboardEntry {
    pub server: MonitorServer,
    pub instances: Vec<InstanceDashboardEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceDashboardEntry {
    pub instance: ServiceInstance,
    pub latest_status: Option<ServiceStatus>,
}

/// Query parameters for history endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<i64>,
}

/// Server detail with its instances and their latest statuses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDetail {
    pub server: MonitorServer,
    pub instances: Vec<InstanceDashboardEntry>,
}
