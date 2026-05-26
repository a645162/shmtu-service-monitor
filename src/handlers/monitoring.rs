use axum::{extract::State, Json};
use sqlx::SqlitePool;

use crate::db::sqlite;
use crate::models::{DashboardSummary, ServiceDashboardEntry};

/// GET /api/dashboard — Get dashboard summary data.
pub async fn get_dashboard(
    State(pool): State<SqlitePool>,
) -> Result<Json<DashboardSummary>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let services = match sqlite::list_services(&pool).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    };

    let mut entries = Vec::new();
    let mut healthy = 0i64;
    let mut busy = 0i64;
    let mut unavailable = 0i64;

    for svc in &services {
        let latest = sqlite::get_latest_status(&pool, svc.id).await.ok().flatten();

        if let Some(ref st) = latest {
            match st.status.as_str() {
                "healthy" => healthy += 1,
                "busy" => busy += 1,
                _ => unavailable += 1,
            }
        } else {
            unavailable += 1;
        }

        entries.push(ServiceDashboardEntry {
            service: svc.clone(),
            latest_status: latest,
        });
    }

    Ok(Json(DashboardSummary {
        total_services: services.len() as i64,
        healthy_services: healthy,
        busy_services: busy,
        unavailable_services: unavailable,
        services: entries,
    }))
}
