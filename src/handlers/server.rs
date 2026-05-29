use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;

use crate::db::sqlite;
use crate::models::{CreateServerRequest, MonitorServer};

/// POST /api/servers — Create a new server group.
pub async fn create_server(
    State(pool): State<SqlitePool>,
    Json(req): Json<CreateServerRequest>,
) -> Result<(StatusCode, Json<MonitorServer>), (StatusCode, Json<serde_json::Value>)> {
    if req.name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "name is required"})),
        ));
    }

    match sqlite::insert_server(&pool, &req.name, &req.description).await {
        Ok(server) => Ok((StatusCode::CREATED, Json(server))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// GET /api/servers — List all server groups.
pub async fn list_servers(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<MonitorServer>>, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::list_servers(&pool).await {
        Ok(servers) => Ok(Json(servers)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// GET /api/servers/:id — Get server group detail.
pub async fn get_server(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<MonitorServer>, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::get_server(&pool, id).await {
        Ok(Some(server)) => Ok(Json(server)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Server not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}

/// DELETE /api/servers/:id — Delete a server group (and its instances).
pub async fn delete_server(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match sqlite::delete_server(&pool, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Server not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )),
    }
}
