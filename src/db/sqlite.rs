use sqlx::SqlitePool;

use crate::models::{MonitorServer, ServiceInstance, ServiceStatus};

/// Create the database tables if they don't exist.
pub async fn init_db(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS monitor_servers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS service_instances (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            server_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            service_type TEXT NOT NULL,
            base_url TEXT NOT NULL,
            poll_interval_secs INTEGER NOT NULL DEFAULT 10,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (server_id) REFERENCES monitor_servers(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS service_statuses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            instance_id INTEGER NOT NULL,
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
            FOREIGN KEY (instance_id) REFERENCES service_instances(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Migrate from old tables if they exist
    migrate_from_v1(pool).await?;

    // Create indexes
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_instance_server_id ON service_instances(server_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_status_instance_id ON service_statuses(instance_id)",
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

/// Migrate data from old v1 schema (flat `services` table) to v2 (server groups).
async fn migrate_from_v1(pool: &SqlitePool) -> anyhow::Result<()> {
    let old_exists: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='services'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !old_exists {
        return Ok(());
    }

    let new_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM monitor_servers")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    if new_count > 0 {
        sqlx::query("DROP TABLE IF EXISTS service_statuses")
            .execute(pool)
            .await?;
        sqlx::query("DROP TABLE IF EXISTS services")
            .execute(pool)
            .await?;
        return Ok(());
    }

    tracing::info!("Migrating from v1 schema to v2 (multi-server)...");

    let default_server = insert_server(pool, "Default Server", "Auto-migrated from v1").await?;

    let migrated = sqlx::query(
        r#"
        INSERT INTO service_instances (id, server_id, name, service_type, base_url, poll_interval_secs, created_at)
        SELECT id, ?, name, service_type, base_url, poll_interval_secs, created_at
        FROM services
        "#,
    )
    .bind(default_server.id)
    .execute(pool)
    .await?;

    tracing::info!("Migrated {} services to instances", migrated.rows_affected());

    // Rename old status table, migrate data
    sqlx::query("ALTER TABLE service_statuses RENAME TO service_statuses_old")
        .execute(pool)
        .await
        .ok();

    // Recreate the status table with new schema
    sqlx::query(
        r#"
        CREATE TABLE service_statuses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            instance_id INTEGER NOT NULL,
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
            FOREIGN KEY (instance_id) REFERENCES service_instances(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO service_statuses (instance_id, status, availability_level, models_loaded,
            pending_requests, queue_capacity, utilization_percent,
            avg_response_ms, total_requests, success_count, failure_count,
            polled_at, response_time_ms)
        SELECT service_id, status, availability_level, models_loaded,
            pending_requests, queue_capacity, utilization_percent,
            avg_response_ms, total_requests, success_count, failure_count,
            polled_at, response_time_ms
        FROM service_statuses_old
        "#,
    )
    .execute(pool)
    .await
    .ok();

    sqlx::query("DROP TABLE IF EXISTS service_statuses_old")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS services")
        .execute(pool)
        .await?;

    tracing::info!("V1 → V2 migration complete");
    Ok(())
}

// ── Server (server group) CRUD ──

pub async fn insert_server(
    pool: &SqlitePool,
    name: &str,
    description: &str,
) -> anyhow::Result<MonitorServer> {
    let row = sqlx::query_as::<_, MonitorServer>(
        r#"
        INSERT INTO monitor_servers (name, description)
        VALUES (?, ?)
        RETURNING id, name, description, created_at
        "#,
    )
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_servers(pool: &SqlitePool) -> anyhow::Result<Vec<MonitorServer>> {
    let rows = sqlx::query_as::<_, MonitorServer>(
        "SELECT id, name, description, created_at FROM monitor_servers ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_server(pool: &SqlitePool, id: i64) -> anyhow::Result<Option<MonitorServer>> {
    let row = sqlx::query_as::<_, MonitorServer>(
        "SELECT id, name, description, created_at FROM monitor_servers WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn delete_server(pool: &SqlitePool, id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM monitor_servers WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ── Service Instance CRUD ──

pub async fn insert_instance(
    pool: &SqlitePool,
    server_id: i64,
    name: &str,
    service_type: &str,
    base_url: &str,
    poll_interval_secs: i64,
) -> anyhow::Result<ServiceInstance> {
    let row = sqlx::query_as::<_, ServiceInstance>(
        r#"
        INSERT INTO service_instances (server_id, name, service_type, base_url, poll_interval_secs)
        VALUES (?, ?, ?, ?, ?)
        RETURNING id, server_id, name, service_type, base_url, poll_interval_secs, created_at
        "#,
    )
    .bind(server_id)
    .bind(name)
    .bind(service_type)
    .bind(base_url)
    .bind(poll_interval_secs as i64)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_instances_by_server(
    pool: &SqlitePool,
    server_id: i64,
) -> anyhow::Result<Vec<ServiceInstance>> {
    let rows = sqlx::query_as::<_, ServiceInstance>(
        "SELECT id, server_id, name, service_type, base_url, poll_interval_secs, created_at FROM service_instances WHERE server_id = ? ORDER BY id",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_all_instances(pool: &SqlitePool) -> anyhow::Result<Vec<ServiceInstance>> {
    let rows = sqlx::query_as::<_, ServiceInstance>(
        "SELECT id, server_id, name, service_type, base_url, poll_interval_secs, created_at FROM service_instances ORDER BY server_id, id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_instance(pool: &SqlitePool, id: i64) -> anyhow::Result<Option<ServiceInstance>> {
    let row = sqlx::query_as::<_, ServiceInstance>(
        "SELECT id, server_id, name, service_type, base_url, poll_interval_secs, created_at FROM service_instances WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn delete_instance(pool: &SqlitePool, id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM service_instances WHERE id = ?")
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
            instance_id, status, availability_level, models_loaded,
            pending_requests, queue_capacity, utilization_percent,
            avg_response_ms, total_requests, success_count, failure_count,
            polled_at, response_time_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(status.instance_id)
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
    instance_id: i64,
) -> anyhow::Result<Option<ServiceStatus>> {
    let row = sqlx::query_as::<_, ServiceStatus>(
        r#"
        SELECT id, instance_id, status, availability_level, models_loaded,
               pending_requests, queue_capacity, utilization_percent,
               avg_response_ms, total_requests, success_count, failure_count,
               polled_at, response_time_ms
        FROM service_statuses
        WHERE instance_id = ?
        ORDER BY polled_at DESC
        LIMIT 1
        "#,
    )
    .bind(instance_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn get_status_history(
    pool: &SqlitePool,
    instance_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
) -> anyhow::Result<Vec<ServiceStatus>> {
    let mut query = String::from(
        r#"
        SELECT id, instance_id, status, availability_level, models_loaded,
               pending_requests, queue_capacity, utilization_percent,
               avg_response_ms, total_requests, success_count, failure_count,
               polled_at, response_time_ms
        FROM service_statuses
        WHERE instance_id = ?
        "#,
    );

    if from.is_some() {
        query.push_str(" AND polled_at >= ?");
    }
    if to.is_some() {
        query.push_str(" AND polled_at <= ?");
    }

    query.push_str(" ORDER BY polled_at DESC LIMIT ?");

    let mut q = sqlx::query_as::<_, ServiceStatus>(&query).bind(instance_id);

    if let Some(f) = from {
        q = q.bind(f);
    }
    if let Some(t) = to {
        q = q.bind(t);
    }

    let rows = q.bind(limit).fetch_all(pool).await?;
    Ok(rows)
}

/// Delete old status records, keeping only the most recent N per instance.
pub async fn cleanup_old_statuses(pool: &SqlitePool, keep_count: i64) -> anyhow::Result<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM service_statuses
        WHERE id NOT IN (
            SELECT id FROM service_statuses s2
            WHERE s2.instance_id = service_statuses.instance_id
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
