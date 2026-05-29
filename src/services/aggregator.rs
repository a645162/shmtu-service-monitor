use sqlx::SqlitePool;

use crate::db::sqlite;

/// Aggregates statistics for an instance over a given time window.
pub struct InstanceAggregation {
    pub total_polls: i64,
    pub healthy_count: i64,
    pub unavailable_count: i64,
    pub avg_response_time_ms: f64,
    pub avg_utilization_percent: f64,
    pub uptime_percent: f64,
}

/// Compute aggregate statistics for an instance from its recent status history.
pub async fn aggregate_instance_stats(
    pool: &SqlitePool,
    instance_id: i64,
    limit: i64,
) -> anyhow::Result<InstanceAggregation> {
    let history = sqlite::get_status_history(pool, instance_id, None, None, limit).await?;

    let total_polls = history.len() as i64;
    if total_polls == 0 {
        return Ok(InstanceAggregation {
            total_polls: 0,
            healthy_count: 0,
            unavailable_count: 0,
            avg_response_time_ms: 0.0,
            avg_utilization_percent: 0.0,
            uptime_percent: 0.0,
        });
    }

    let healthy_count = history
        .iter()
        .filter(|s| s.status == "healthy")
        .count() as i64;

    let unavailable_count = history
        .iter()
        .filter(|s| s.status == "unavailable")
        .count() as i64;

    let avg_response_time_ms = history.iter().map(|s| s.response_time_ms).sum::<f64>()
        / total_polls as f64;

    let avg_utilization_percent =
        history.iter().map(|s| s.utilization_percent).sum::<f64>() / total_polls as f64;

    let uptime_percent = (healthy_count as f64 / total_polls as f64) * 100.0;

    Ok(InstanceAggregation {
        total_polls,
        healthy_count,
        unavailable_count,
        avg_response_time_ms,
        avg_utilization_percent,
        uptime_percent,
    })
}
