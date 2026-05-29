use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;

use crate::db::sqlite;
use crate::models::{CreateInstanceRequest, HistoryQuery, ServiceInstance, ServiceStatus};

/// POST /api/servers/:server_id/instances — Register a new instance under a server.
pub async fn register_instance(
    State(pool): State<SqlitePool>,
    Path(server_id): Path<i64>,
    Json(req): Json<CreateInstanceRequest>,
) -> Result<(StatusCode, Json<ServiceInstance>), (StatusCode, Json<serde_json::Value>)> {
    // Verify server exists
    match sqlite::get_server(&pool, server_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Server not found"})),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }

    if req.base_url.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "base_url is required"})),
        ));
    }

    let valid_types = ["dotnet-ocr", "cpp-ocr", "rust-ocr"];
    if !valid_types.contains(&req.service_type.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Invalid service_type. Must be one of: {:?}", valid_types)
            })),
        ));
    }

    match sqlite::insert_instance(
        &pool,
        server_id,
        &req.name,
        &req.service_type,
        &req.base_url,
        req.poll_interval_secs,
    )
    .await
    {
        Ok(instance) => Ok((StatusCode::CREATED, Json(instance))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// GET /api/servers/:server_id/instances — List instances under a server.
pub async fn list_instances(
    State(pool): State<SqlitePool>,
    Path(server_id): Path<i64>,
) -> Result<Json<Vec<ServiceInstance>>, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::list_instances_by_server(&pool, server_id).await {
        Ok(instances) => Ok(Json(instances)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// GET /api/instances/:id — Get instance detail.
pub async fn get_instance(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ServiceInstance>, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::get_instance(&pool, id).await {
        Ok(Some(instance)) => Ok(Json(instance)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Instance not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// DELETE /api/instances/:id — Delete an instance.
pub async fn delete_instance(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::delete_instance(&pool, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Instance not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// GET /api/instances/:id/status — Get the latest status for an instance.
pub async fn get_instance_status(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ServiceStatus>, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::get_instance(&pool, id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Instance not found"})),
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
            Json(serde_json::json!({"error": "No status data available for this instance"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// GET /api/instances/:id/history — Get historical statuses for an instance.
pub async fn get_instance_history(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<ServiceStatus>>, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::get_instance(&pool, id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Instance not found"})),
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

    match sqlite::get_status_history(&pool, id, query.from.as_deref(), query.to.as_deref(), limit)
        .await
    {
        Ok(history) => Ok(Json(history)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}
