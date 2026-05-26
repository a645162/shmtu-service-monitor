use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A registered service to be monitored.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Service {
    pub id: i64,
    pub name: String,
    pub service_type: String, // "dotnet-ocr", "cpp-ocr", "rust-ocr"
    pub base_url: String,     // e.g. "http://192.168.1.10:21600"
    pub poll_interval_secs: i64,
    pub created_at: DateTime<Utc>,
}

/// Request body for registering a new service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceRequest {
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
    pub service_id: i64,
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
/// This mirrors the dotnet OCR server's status response format.
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
}

/// Dashboard summary data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_services: i64,
    pub healthy_services: i64,
    pub busy_services: i64,
    pub unavailable_services: i64,
    pub services: Vec<ServiceDashboardEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDashboardEntry {
    pub service: Service,
    pub latest_status: Option<ServiceStatus>,
}

/// Query parameters for history endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryQuery {
    pub from: Option<String>, // ISO 8601 datetime
    pub to: Option<String>,
    pub limit: Option<i64>,
}
