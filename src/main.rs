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
use handlers::{instance, monitoring, server};
use services::poller::Poller;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = AppConfig::from_env();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    init_db(&pool).await?;

    let poller = Poller::new(pool.clone(), config.poll_interval_secs);
    let poller_handle = tokio::spawn(async move {
        poller.run().await;
    });

    // Server group routes
    let server_routes = Router::new()
        .route("/", axum::routing::post(server::create_server))
        .route("/", axum::routing::get(server::list_servers))
        .route("/{id}", axum::routing::get(server::get_server))
        .route("/{id}", axum::routing::delete(server::delete_server))
        .route("/{id}/detail", axum::routing::get(monitoring::get_server_detail))
        .route("/{server_id}/instances", axum::routing::post(instance::register_instance))
        .route("/{server_id}/instances", axum::routing::get(instance::list_instances))
        .with_state(pool.clone());

    // Instance routes
    let instance_routes = Router::new()
        .route("/{id}", axum::routing::get(instance::get_instance))
        .route("/{id}", axum::routing::delete(instance::delete_instance))
        .route("/{id}/status", axum::routing::get(instance::get_instance_status))
        .route("/{id}/history", axum::routing::get(instance::get_instance_history))
        .with_state(pool.clone());

    // Dashboard
    let dashboard_routes = Router::new()
        .route("/", axum::routing::get(monitoring::get_dashboard))
        .with_state(pool.clone());

    let api_routes = Router::new()
        .nest("/servers", server_routes)
        .nest("/instances", instance_routes)
        .nest("/dashboard", dashboard_routes);

    let app = Router::new()
        .nest("/api", api_routes)
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
