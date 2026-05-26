/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub poll_interval_secs: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("MONITOR_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3100),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:shmtu_monitor.db?mode=rwc".to_string()),
            poll_interval_secs: std::env::var("POLL_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        }
    }
}
