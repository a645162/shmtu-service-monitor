use std::net::SocketAddr;

use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

mod config;
mod db;
mod handlers;
mod models;
mod services;

use config::AppConfig;
use db::sqlite::init_db;
use handlers::{monitoring, service};
use services::poller::Poller;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = AppConfig::from_env();

    // Initialize SQLite pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // Run migrations (create tables)
    init_db(&pool).await?;

    // Start the poller
    let poller = Poller::new(pool.clone(), config.poll_interval_secs);
    let poller_handle = tokio::spawn(async move {
        poller.run().await;
    });

    // Build the API router
    let api_routes = Router::new()
        .route("/services", axum::routing::post(service::register_service))
        .route("/services", axum::routing::get(service::list_services))
        .route("/services/{id}", axum::routing::get(service::get_service))
        .route("/services/{id}", axum::routing::delete(service::delete_service))
        .route("/services/{id}/status", axum::routing::get(service::get_service_status))
        .route("/services/{id}/history", axum::routing::get(service::get_service_history))
        .route("/dashboard", axum::routing::get(monitoring::get_dashboard))
        .with_state(pool.clone());

    let app = Router::new()
        .nest("/api", api_routes)
        // Serve frontend static files in production
        .fallback_service(ServeDir::new("frontend/dist").append_index_html_on_directories(true))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("SHMTU Service Monitor starting on {}", addr);
    tracing::info!("API available at http://{}/api", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    poller_handle.abort();

    Ok(())
}
