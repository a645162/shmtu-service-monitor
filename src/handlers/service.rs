use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;

use crate::db::sqlite;
use crate::models::{CreateServiceRequest, HistoryQuery, Service, ServiceStatus};

/// POST /api/services — Register a new service.
pub async fn register_service(
    State(pool): State<SqlitePool>,
    Json(req): Json<CreateServiceRequest>,
) -> Result<(StatusCode, Json<Service>), (StatusCode, Json<serde_json::Value>)> {
    // Validate base_url
    if req.base_url.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "base_url is required"})),
        ));
    }

    // Validate service_type
    let valid_types = ["dotnet-ocr", "cpp-ocr", "rust-ocr"];
    if !valid_types.contains(&req.service_type.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Invalid service_type. Must be one of: {:?}", valid_types)
            })),
        ));
    }

    match sqlite::insert_service(
        &pool,
        &req.name,
        &req.service_type,
        &req.base_url,
        req.poll_interval_secs,
    )
    .await
    {
        Ok(service) => Ok((StatusCode::CREATED, Json(service))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// GET /api/services — List all services.
pub async fn list_services(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Service>>, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::list_services(&pool).await {
        Ok(services) => Ok(Json(services)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// GET /api/services/:id — Get service detail.
pub async fn get_service(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Service>, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::get_service(&pool, id).await {
        Ok(Some(service)) => Ok(Json(service)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Service not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// DELETE /api/services/:id — Delete a service.
pub async fn delete_service(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::delete_service(&pool, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Service not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// GET /api/services/:id/status — Get the latest status for a service.
pub async fn get_service_status(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ServiceStatus>, (StatusCode, Json<serde_json::Value>)> {
    // Verify service exists
    match sqlite::get_service(&pool, id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Service not found"})),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }

    match sqlite::get_latest_status(&pool, id).await {
        Ok(Some(status)) => Ok(Json(status)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "No status data available for this service"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// GET /api/services/:id/history — Get historical statuses for a service.
pub async fn get_service_history(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<ServiceStatus>>, (StatusCode, Json<serde_json::Value>)> {
    // Verify service exists
    match sqlite::get_service(&pool, id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Service not found"})),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }

    let limit = query.limit.unwrap_or(100).min(1000);

    match sqlite::get_status_history(
        &pool,
        id,
        query.from.as_deref(),
        query.to.as_deref(),
        limit,
    )
    .await
    {
        Ok(history) => Ok(Json(history)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}
