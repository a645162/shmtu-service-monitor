use sqlx::SqlitePool;

use crate::models::{Service, ServiceStatus};

/// Create the database tables if they don't exist.
pub async fn init_db(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS services (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            service_type TEXT NOT NULL,
            base_url TEXT NOT NULL,
            poll_interval_secs INTEGER NOT NULL DEFAULT 10,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS service_statuses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            service_id INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'unavailable',
            availability_level TEXT NOT NULL DEFAULT 'unavailable',
            models_loaded INTEGER NOT NULL DEFAULT 0,
            pending_requests INTEGER NOT NULL DEFAULT 0,
            queue_capacity INTEGER NOT NULL DEFAULT 0,
            utilization_percent REAL NOT NULL DEFAULT 0.0,
            avg_response_ms REAL,
            total_requests INTEGER,
            success_count INTEGER,
            failure_count INTEGER,
            polled_at TEXT NOT NULL DEFAULT (datetime('now')),
            response_time_ms REAL NOT NULL DEFAULT 0.0,
            FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create index for faster history queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_status_service_id ON service_statuses(service_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_status_polled_at ON service_statuses(polled_at)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ── Service CRUD ──

pub async fn insert_service(
    pool: &SqlitePool,
    name: &str,
    service_type: &str,
    base_url: &str,
    poll_interval_secs: i64,
) -> anyhow::Result<Service> {
    let row = sqlx::query_as::<_, Service>(
        r#"
        INSERT INTO services (name, service_type, base_url, poll_interval_secs)
        VALUES (?, ?, ?, ?)
        RETURNING id, name, service_type, base_url, poll_interval_secs, created_at
        "#,
    )
    .bind(name)
    .bind(service_type)
    .bind(base_url)
    .bind(poll_interval_secs as i64)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_services(pool: &SqlitePool) -> anyhow::Result<Vec<Service>> {
    let rows = sqlx::query_as::<_, Service>(
        "SELECT id, name, service_type, base_url, poll_interval_secs, created_at FROM services ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_service(pool: &SqlitePool, id: i64) -> anyhow::Result<Option<Service>> {
    let row = sqlx::query_as::<_, Service>(
        "SELECT id, name, service_type, base_url, poll_interval_secs, created_at FROM services WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn delete_service(pool: &SqlitePool, id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM services WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ── Status operations ──

pub async fn insert_status(pool: &SqlitePool, status: &ServiceStatus) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO service_statuses (
            service_id, status, availability_level, models_loaded,
            pending_requests, queue_capacity, utilization_percent,
            avg_response_ms, total_requests, success_count, failure_count,
            polled_at, response_time_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(status.service_id)
    .bind(&status.status)
    .bind(&status.availability_level)
    .bind(status.models_loaded as i32)
    .bind(status.pending_requests)
    .bind(status.queue_capacity)
    .bind(status.utilization_percent)
    .bind(status.avg_response_ms)
    .bind(status.total_requests)
    .bind(status.success_count)
    .bind(status.failure_count)
    .bind(status.polled_at.to_rfc3339())
    .bind(status.response_time_ms)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_latest_status(
    pool: &SqlitePool,
    service_id: i64,
) -> anyhow::Result<Option<ServiceStatus>> {
    let row = sqlx::query_as::<_, ServiceStatus>(
        r#"
        SELECT id, service_id, status, availability_level, models_loaded,
               pending_requests, queue_capacity, utilization_percent,
               avg_response_ms, total_requests, success_count, failure_count,
               polled_at, response_time_ms
        FROM service_statuses
        WHERE service_id = ?
        ORDER BY polled_at DESC
        LIMIT 1
        "#,
    )
    .bind(service_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn get_status_history(
    pool: &SqlitePool,
    service_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
) -> anyhow::Result<Vec<ServiceStatus>> {
    let mut query = String::from(
        r#"
        SELECT id, service_id, status, availability_level, models_loaded,
               pending_requests, queue_capacity, utilization_percent,
               avg_response_ms, total_requests, success_count, failure_count,
               polled_at, response_time_ms
        FROM service_statuses
        WHERE service_id = ?
        "#,
    );

    if from.is_some() {
        query.push_str(" AND polled_at >= ?");
    }
    if to.is_some() {
        query.push_str(" AND polled_at <= ?");
    }

    query.push_str(" ORDER BY polled_at DESC LIMIT ?");

    let mut q = sqlx::query_as::<_, ServiceStatus>(&query).bind(service_id);

    if let Some(f) = from {
        q = q.bind(f);
    }
    if let Some(t) = to {
        q = q.bind(t);
    }

    let rows = q.bind(limit).fetch_all(pool).await?;
    Ok(rows)
}

/// Delete old status records, keeping only the most recent N per service.
pub async fn cleanup_old_statuses(pool: &SqlitePool, keep_count: i64) -> anyhow::Result<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM service_statuses
        WHERE id NOT IN (
            SELECT id FROM service_statuses s2
            WHERE s2.service_id = service_statuses.service_id
            ORDER BY polled_at DESC
            LIMIT ?
        )
        "#,
    )
    .bind(keep_count)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
