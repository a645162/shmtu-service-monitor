use axum::{extract::State, Json};
use sqlx::SqlitePool;

use crate::db::sqlite;
use crate::models::{
    DashboardSummary, InstanceDashboardEntry, ServerDashboardEntry, ServerDetail,
};

/// GET /api/dashboard — Get dashboard summary data grouped by servers.
pub async fn get_dashboard(
    State(pool): State<SqlitePool>,
) -> Result<Json<DashboardSummary>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let servers = match sqlite::list_servers(&pool).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    };

    let mut server_entries = Vec::new();
    let mut total_instances = 0i64;
    let mut healthy = 0i64;
    let mut busy = 0i64;
    let mut unavailable = 0i64;

    for srv in &servers {
        let instances = match sqlite::list_instances_by_server(&pool, srv.id).await {
            Ok(v) => v,
            Err(e) => {
                return Err((
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                ))
            }
        };

        let mut instance_entries = Vec::new();

        for inst in &instances {
            total_instances += 1;
            let latest = sqlite::get_latest_status(&pool, inst.id).await.ok().flatten();

            if let Some(ref st) = latest {
                match st.status.as_str() {
                    "healthy" => healthy += 1,
                    "busy" => busy += 1,
                    _ => unavailable += 1,
                }
            } else {
                unavailable += 1;
            }

            instance_entries.push(InstanceDashboardEntry {
                instance: inst.clone(),
                latest_status: latest,
            });
        }

        server_entries.push(ServerDashboardEntry {
            server: srv.clone(),
            instances: instance_entries,
        });
    }

    Ok(Json(DashboardSummary {
        total_servers: servers.len() as i64,
        total_instances,
        healthy_instances: healthy,
        busy_instances: busy,
        unavailable_instances: unavailable,
        servers: server_entries,
    }))
}

/// GET /api/servers/:id/detail — Get server detail with instances and their latest statuses.
pub async fn get_server_detail(
    State(pool): State<SqlitePool>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<ServerDetail>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let server = match sqlite::get_server(&pool, id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Server not found"})),
            ))
        }
        Err(e) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    };

    let instances = match sqlite::list_instances_by_server(&pool, id).await {
        Ok(v) => v,
        Err(e) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    };

    let mut instance_entries = Vec::new();
    for inst in &instances {
        let latest = sqlite::get_latest_status(&pool, inst.id).await.ok().flatten();
        instance_entries.push(InstanceDashboardEntry {
            instance: inst.clone(),
            latest_status: latest,
        });
    }

    Ok(Json(ServerDetail {
        server,
        instances: instance_entries,
    }))
}
