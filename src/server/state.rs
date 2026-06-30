//! ############################################################################
//! @file       state.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      SQLite state store — initialization, migrations, and CRUD operations.
//!
//! @details
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-02 | 1.0.0   | Phaneendra Bhattiprolu  | Initial implementation. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! ############################################################################
//! State store — persists scan history, schedules, and targets.
//!
//! Uses sqlx::SqlitePool — SQLite-backed state store.
//! URL config in qxscan.toml [database] section.
//!
//! Database URL (default):
//!   sqlite://~/.qxscan/state.db
//!
//! Credentials via env var expansion: postgres://${DB_USER}:${DB_PASS}@host/db

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use sqlx::{ConnectOptions, Row, SqlitePool};
use uuid::Uuid;

use crate::qem::metadata::ScanEvent;

/// Expand ${VAR_NAME} and $VAR_NAME patterns with environment variable values.
/// The expanded URL is cached at config load time — never re-reads env vars
/// on every query.
pub fn expand_env_vars(url: &str) -> String {
    let mut result = String::with_capacity(url.len());
    let mut chars = url.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            let mut var_name = String::new();
            if chars.peek() == Some(&{ '{' }) {
                // ${VAR_NAME} syntax
                chars.next(); // skip '{'
                for ch in chars.by_ref() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
            } else {
                // $VAR_NAME syntax (stop at non-alphanumeric/underscore)
                for ch in chars.by_ref() {
                    if ch.is_alphanumeric() || ch == '_' {
                        var_name.push(ch);
                    } else {
                        // Put back the non-var char (we already consumed it, so
                        // we need to re-add the rest of the URL)
                        result.push(ch);
                        break;
                    }
                }
            }
            let value = std::env::var(&var_name).unwrap_or_else(|_| {
                log::warn!("environment variable ${var_name} is not set; leaving unexpanded");
                format!("${{{var_name}}}")
            });
            result.push_str(&value);
        } else {
            result.push(c);
        }
    }

    result
}

/// Mask the password portion of a database URL for safe logging.
/// e.g. "postgres://user:secret@host/db" → "postgres://user:***@host/db"
pub fn mask_password(url: &str) -> String {
    if let Some(after_scheme) = url.split("://").nth(1) {
        if let Some(at_pos) = after_scheme.find('@') {
            if let Some(colon_pos) = after_scheme[..at_pos].find(':') {
                let user = &after_scheme[..colon_pos];
                let scheme_end = url.find("://").unwrap_or(0) + 3;
                let after_host = &url[scheme_end + at_pos + 1..];
                let scheme = &url[..scheme_end - 3];
                return format!("{scheme}://{user}:***@{after_host}");
            }
        }
    }
    url.to_string()
}

/// Default SQLite database path.
fn default_db_path() -> PathBuf {
    std::env::var("QXSCAN_STATE_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(".qxscan").join("state.db"))
                .unwrap_or_else(|| PathBuf::from("/var/lib/qxscan/state.db"))
        })
}

/// Build the default SQLite URL from the state file path.
pub fn default_db_url() -> String {
    let path = default_db_path();
    format!("sqlite://{}", path.display())
}

/// Initialize the connection pool and run migrations.
pub async fn init_pool(url: &str) -> anyhow::Result<SqlitePool> {
    let expanded = expand_env_vars(url);

    let masked = mask_password(&expanded);
    log::info!("connecting to database: {masked}");

    // Extract file path from sqlite:// URL
    let path = expanded.strip_prefix("sqlite://").unwrap_or(&expanded);

    // Ensure the parent directory exists
    if !path.is_empty() && !path.contains(":memory:") {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!(
                    "cannot create database directory '{}': {e}",
                    parent.display()
                )
            })?;
        }
    }

    let opts = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .log_statements(log::LevelFilter::Debug)
        .log_slow_statements(log::LevelFilter::Warn, std::time::Duration::from_secs(5));

    let pool = SqlitePool::connect_with(opts).await?;

    // Enable WAL mode
    let _ = sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    log::info!("database connected successfully");

    Ok(pool)
}

// ─── ScanEvent CRUD ─────────────────────────────────────────

/// Insert a scan event into the database.
pub async fn insert_scan_event(pool: &SqlitePool, event: &ScanEvent) -> anyhow::Result<()> {
    let json = serde_json::to_string(event)?;
    let status = serde_json::to_value(&event.overall_status)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "unknown".into());

    sqlx::query(
        "INSERT OR REPLACE INTO scan_events \
         (id, scanned_at, target_host, target_port, service, status, event_json) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(event.scan_id.to_string())
    .bind(event.scanned_at.to_rfc3339())
    .bind(&event.target.host)
    .bind(event.target.port as i64)
    .bind(&event.target.service)
    .bind(&status)
    .bind(&json)
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetch recent scan events, newest first.
pub async fn get_recent_scans(pool: &SqlitePool, limit: usize) -> anyhow::Result<Vec<ScanEvent>> {
    let rows = sqlx::query("SELECT event_json FROM scan_events ORDER BY scanned_at DESC LIMIT ?")
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

    let mut events = Vec::new();
    for row in rows {
        let json: String = row.get(0);
        if let Ok(event) = serde_json::from_str(&json) {
            events.push(event);
        }
    }
    Ok(events)
}

// ─── Schedule CRUD ──────────────────────────────────────────

/// A periodic scan schedule.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Schedule {
    pub id: Uuid,
    pub label: Option<String>,
    pub targets_file: PathBuf,
    pub cron_expr: String,
    pub standards: Vec<String>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub enabled: bool,
}

impl Schedule {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> anyhow::Result<Self> {
        let id: String = row.try_get("id")?;
        let label: Option<String> = row.try_get("label").ok().flatten();
        let targets_file: String = row.try_get("targets_file")?;
        let cron_expr: String = row.try_get("cron_expr")?;
        let standards: String = row.try_get("standards")?;
        let last_run_at: Option<String> = row.try_get("last_run_at").ok().flatten();
        let next_run_at: String = row.try_get("next_run_at")?;
        let enabled_val: i64 = row.try_get("enabled")?;

        Ok(Self {
            id: Uuid::parse_str(&id)
                .map_err(|e| anyhow::anyhow!("invalid schedule UUID '{id}': {e}"))?,
            label,
            targets_file: PathBuf::from(targets_file),
            cron_expr,
            standards: standards.split(',').map(String::from).collect(),
            last_run_at: last_run_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
            next_run_at: DateTime::parse_from_rfc3339(&next_run_at)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            enabled: enabled_val != 0,
        })
    }
}

/// Insert a new schedule.
pub async fn insert_schedule(pool: &SqlitePool, s: &Schedule) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO schedules \
         (id, label, targets_file, cron_expr, standards, last_run_at, next_run_at, enabled) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(s.id.to_string())
    .bind(&s.label)
    .bind(s.targets_file.to_string_lossy().to_string())
    .bind(&s.cron_expr)
    .bind(s.standards.join(","))
    .bind(s.last_run_at.map(|d| d.to_rfc3339()))
    .bind(s.next_run_at.to_rfc3339())
    .bind(s.enabled as i64)
    .execute(pool)
    .await?;
    Ok(())
}

/// List all schedules, ordered by next run time.
pub async fn list_schedules(pool: &SqlitePool) -> anyhow::Result<Vec<Schedule>> {
    let rows = sqlx::query(
        "SELECT id, label, targets_file, cron_expr, standards, last_run_at, next_run_at, enabled \
         FROM schedules ORDER BY next_run_at ASC",
    )
    .fetch_all(pool)
    .await?;

    let mut schedules = Vec::new();
    for row in rows {
        schedules.push(Schedule::from_row(&row)?);
    }
    Ok(schedules)
}

/// Delete a schedule by ID. Returns true if a row was deleted.
pub async fn delete_schedule(pool: &SqlitePool, id: &str) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM schedules WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Fetch all due (enabled, next_run_at <= now) schedules.
pub async fn get_due_schedules(pool: &SqlitePool) -> anyhow::Result<Vec<Schedule>> {
    let now = Utc::now().to_rfc3339();
    let rows = sqlx::query(
        "SELECT id, label, targets_file, cron_expr, standards, last_run_at, next_run_at, enabled \
         FROM schedules WHERE enabled = 1 AND next_run_at <= ? \
         ORDER BY next_run_at ASC",
    )
    .bind(&now)
    .fetch_all(pool)
    .await?;

    let mut schedules = Vec::new();
    for row in rows {
        schedules.push(Schedule::from_row(&row)?);
    }
    Ok(schedules)
}

/// Update last_run_at and next_run_at for a schedule.
pub async fn update_next_run(
    pool: &SqlitePool,
    id: &str,
    next_run_at: &DateTime<Utc>,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE schedules SET last_run_at = ?, next_run_at = ? WHERE id = ?")
        .bind(Utc::now().to_rfc3339())
        .bind(next_run_at.to_rfc3339())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ─── ScanReport CRUD ──────────────────────────────────────────

/// A rendered scan report stored in the database.
/// Use this to retrieve reports for export without re-scanning.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanReport {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub format: String,
    pub content: String,
    pub file_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Save a rendered report (HTML, JSON, etc.) to the database for later retrieval.
pub async fn insert_scan_report(
    pool: &SqlitePool,
    scan_id: &str,
    format: &str,
    content: &str,
    file_path: Option<&str>,
) -> anyhow::Result<Uuid> {
    let id = Uuid::new_v4();
    let created_at = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO scan_reports \
         (id, scan_id, format, content, file_path, created_at) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(scan_id)
    .bind(format)
    .bind(content)
    .bind(file_path)
    .bind(&created_at)
    .execute(pool)
    .await?;

    Ok(id)
}

/// Retrieve scan reports, optionally filtered by scan_id and/or format.
/// Ordered by creation time, newest first.
pub async fn get_scan_reports(
    pool: &SqlitePool,
    scan_id: Option<&str>,
    format: Option<&str>,
    limit: usize,
) -> anyhow::Result<Vec<ScanReport>> {
    let mut sql = String::from(
        "SELECT id, scan_id, format, content, file_path, created_at \
         FROM scan_reports WHERE 1=1",
    );
    if scan_id.is_some() {
        sql.push_str(" AND scan_id = ?");
    }
    if format.is_some() {
        sql.push_str(" AND format = ?");
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT ?");

    let mut query = sqlx::query(&sql);
    if let Some(sid) = scan_id {
        query = query.bind(sid);
    }
    if let Some(fmt) = format {
        query = query.bind(fmt);
    }
    query = query.bind(limit as i64);

    let rows = query.fetch_all(pool).await?;

    let mut reports = Vec::with_capacity(rows.len());
    for row in &rows {
        reports.push(parse_scan_report_row(row)?);
    }
    Ok(reports)
}

/// Retrieve a single scan report by its ID (full UUID or prefix match).
/// Tries exact match first, then falls back to prefix matching for short IDs
/// (e.g. the truncated UUIDs shown in `report list` output).
pub async fn get_scan_report_by_id(
    pool: &SqlitePool,
    id: &str,
) -> anyhow::Result<Option<ScanReport>> {
    // Try exact match first
    let rows = sqlx::query(
        "SELECT id, scan_id, format, content, file_path, created_at \
         FROM scan_reports WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    if let Some(row) = rows.first() {
        return Ok(Some(parse_scan_report_row(row)?));
    }

    // Fall back to prefix match (for short UUID prefixes from `report list`)
    let rows = sqlx::query(
        "SELECT id, scan_id, format, content, file_path, created_at \
         FROM scan_reports WHERE id LIKE ? LIMIT 1",
    )
    .bind(format!("{}%", id))
    .fetch_all(pool)
    .await?;

    Ok(rows.first().and_then(|row| parse_scan_report_row(row).ok()))
}

/// Parse a single ScanReport from a sqlx row.
fn parse_scan_report_row(row: &sqlx::sqlite::SqliteRow) -> anyhow::Result<ScanReport> {
    let id_str: String = row.try_get("id")?;
    let scan_id_str: String = row.try_get("scan_id")?;
    let format_str: String = row.try_get("format")?;
    let content_str: String = row.try_get("content")?;
    let file_path_str: Option<String> = row.try_get("file_path").ok().flatten();
    let created_at_str: String = row.try_get("created_at")?;

    Ok(ScanReport {
        id: Uuid::parse_str(&id_str)
            .map_err(|e| anyhow::anyhow!("invalid report UUID '{id_str}': {e}"))?,
        scan_id: Uuid::parse_str(&scan_id_str)
            .map_err(|e| anyhow::anyhow!("invalid report scan_id '{scan_id_str}': {e}"))?,
        format: format_str,
        content: content_str,
        file_path: file_path_str,
        created_at: DateTime::parse_from_rfc3339(&created_at_str)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

/// Delete a scan report by ID. Returns true if a row was deleted.
pub async fn delete_scan_report(pool: &SqlitePool, id: &str) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM scan_reports WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a single-connection in-memory SQLite pool with the scan_reports table for testing.
    /// Uses max_connections(1) because SQLite `:memory:` databases are per-connection —
    /// a single connection ensures all queries within a test share the same database.
    async fn test_pool() -> SqlitePool {
        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS scan_reports (
                id            TEXT    NOT NULL PRIMARY KEY,
                scan_id       TEXT    NOT NULL,
                format        TEXT    NOT NULL,
                content       TEXT    NOT NULL,
                file_path     TEXT,
                created_at    TEXT    NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_insert_and_list_reports() {
        let pool = test_pool().await;
        let scan_id = Uuid::new_v4();

        let id1 = insert_scan_report(
            &pool,
            &scan_id.to_string(),
            "json",
            r#"{"status":"pass"}"#,
            None,
        )
        .await
        .unwrap();
        let id2 = insert_scan_report(
            &pool,
            &scan_id.to_string(),
            "html",
            "<html></html>",
            Some("/tmp/report.html"),
        )
        .await
        .unwrap();

        let reports = get_scan_reports(&pool, None, None, 10).await.unwrap();
        assert_eq!(reports.len(), 2);

        // Newest first (id2 inserted after id1)
        assert_eq!(reports[0].id, id2);
        assert_eq!(reports[1].id, id1);
    }

    #[tokio::test]
    async fn test_get_reports_filtered_by_scan_id() {
        let pool = test_pool().await;
        let scan_a = Uuid::new_v4();
        let scan_b = Uuid::new_v4();

        insert_scan_report(&pool, &scan_a.to_string(), "json", "a", None)
            .await
            .unwrap();
        insert_scan_report(&pool, &scan_b.to_string(), "json", "b", None)
            .await
            .unwrap();

        let reports = get_scan_reports(&pool, Some(&scan_a.to_string()), None, 10)
            .await
            .unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].content, "a");
    }

    #[tokio::test]
    async fn test_get_reports_filtered_by_format() {
        let pool = test_pool().await;
        let scan_id = Uuid::new_v4();

        insert_scan_report(&pool, &scan_id.to_string(), "json", "{}", None)
            .await
            .unwrap();
        insert_scan_report(&pool, &scan_id.to_string(), "html", "<h1>", None)
            .await
            .unwrap();

        let reports = get_scan_reports(&pool, None, Some("html"), 10)
            .await
            .unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].format, "html");
    }

    #[tokio::test]
    async fn test_get_report_by_id_exact() {
        let pool = test_pool().await;
        let scan_id = Uuid::new_v4();

        let id = insert_scan_report(&pool, &scan_id.to_string(), "json", "exact-match", None)
            .await
            .unwrap();

        let found = get_scan_report_by_id(&pool, &id.to_string()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().content, "exact-match");
    }

    #[tokio::test]
    async fn test_get_report_by_id_prefix() {
        let pool = test_pool().await;
        let scan_id = Uuid::new_v4();

        let id = insert_scan_report(&pool, &scan_id.to_string(), "json", "prefix-match", None)
            .await
            .unwrap();

        let id_str = id.to_string();
        let prefix = &id_str[..8]; // e.g. "a2836ebc"

        let found = get_scan_report_by_id(&pool, prefix).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().content, "prefix-match");
    }

    #[tokio::test]
    async fn test_get_report_by_id_not_found() {
        let pool = test_pool().await;

        let found = get_scan_report_by_id(&pool, "00000000-0000-0000-0000-000000000000")
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_report() {
        let pool = test_pool().await;
        let scan_id = Uuid::new_v4();

        let id = insert_scan_report(&pool, &scan_id.to_string(), "json", "to-delete", None)
            .await
            .unwrap();

        let deleted = delete_scan_report(&pool, &id.to_string()).await.unwrap();
        assert!(deleted);

        let remaining = get_scan_reports(&pool, None, None, 10).await.unwrap();
        assert_eq!(remaining.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_report_not_found() {
        let pool = test_pool().await;

        let deleted = delete_scan_report(&pool, "00000000-0000-0000-0000-000000000000")
            .await
            .unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_get_reports_filtered_by_scan_id_and_format() {
        let pool = test_pool().await;
        let scan_a = Uuid::new_v4();
        let scan_b = Uuid::new_v4();

        // Two reports for scan_a (json + html), one report for scan_b (json)
        insert_scan_report(&pool, &scan_a.to_string(), "json", "{}", None)
            .await
            .unwrap();
        insert_scan_report(&pool, &scan_a.to_string(), "html", "<h1>", None)
            .await
            .unwrap();
        insert_scan_report(&pool, &scan_b.to_string(), "json", "{}", None)
            .await
            .unwrap();

        // Filter by scan_a + html → should return exactly 1 report
        let reports = get_scan_reports(&pool, Some(&scan_a.to_string()), Some("html"), 10)
            .await
            .unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].format, "html");
        assert_eq!(reports[0].scan_id, scan_a);

        // Filter by scan_a + json → should return exactly 1 report
        let reports = get_scan_reports(&pool, Some(&scan_a.to_string()), Some("json"), 10)
            .await
            .unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].format, "json");

        // Filter by scan_b + html → should return 0 (scan_b has no html report)
        let reports = get_scan_reports(&pool, Some(&scan_b.to_string()), Some("html"), 10)
            .await
            .unwrap();
        assert_eq!(reports.len(), 0);
    }
}
