# QXScan OSS — Database Layer Design

## Config (qxscan.toml)

```toml
[database]
# Default: SQLite, zero config, works out of the box.
# Override with postgres:// or mysql:// for shared/enterprise-adjacent setups.
url = "sqlite://~/.qxscan/state.db"

# Optional: connection pool size (ignored for SQLite)
max_connections = 5

# Optional: connection timeout in seconds
connect_timeout_s = 10
```

### URL scheme → driver mapping (auto-detect)

| URL prefix        | Driver     | Crate feature    |
|-------------------|------------|------------------|
| `sqlite://`       | SQLite     | `sqlx/sqlite`    |
| `postgres://`     | PostgreSQL | `sqlx/postgres`  |
| `postgresql://`   | PostgreSQL | `sqlx/postgres`  |
| `mysql://`        | MySQL      | `sqlx/mysql`     |
| `mariadb://`      | MySQL      | `sqlx/mysql`     |

### Credential security rules
- URL in qxscan.toml may reference env vars: `postgres://${DB_USER}:${DB_PASS}@host/db`
- qxscan expands `${VAR}` at startup — never stores the expanded value
- Never pass DB credentials via CLI flags (shell history risk)
- Never log the connection URL (mask password in log output)

---

## Implementation

### Crate: sqlx (not rusqlite for the abstraction layer)

Switch from `rusqlite` direct calls to `sqlx` with the `any` driver.
sqlx's `AnyPool` lets you write one query set that works across all three
drivers. SQLite remains the default and requires zero external dependencies
(bundled via sqlx feature flag).

```toml
# Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = [
    "runtime-tokio-rustls",
    "any",        # AnyPool — driver-agnostic queries
    "sqlite",     # default, always compiled in
] }

[features]
default  = []
postgres = ["sqlx/postgres"]
mysql    = ["sqlx/mysql"]
```

### Why feature-gate PG and MySQL?

Keeps the default binary small — SQLite driver adds ~200K, PostgreSQL
and MySQL clients add ~1–2MB each. OSS users who only need SQLite don't
pay the binary size cost. Users who need PG/MySQL build with:

  cargo build --release --features postgres
  cargo build --release --features postgres,mysql

Document this clearly in README — it's a one-flag build, not a
separate binary.

---

## src/server/state.rs — revised design

```rust
use sqlx::{AnyPool, Row};

pub enum DbDriver { Sqlite, Postgres, Mysql }

pub fn detect_driver(url: &str) -> anyhow::Result<DbDriver> {
    if url.starts_with("sqlite://")              { Ok(DbDriver::Sqlite)   }
    else if url.starts_with("postgres://")
         || url.starts_with("postgresql://")     { Ok(DbDriver::Postgres) }
    else if url.starts_with("mysql://")
         || url.starts_with("mariadb://")        { Ok(DbDriver::Mysql)    }
    else { anyhow::bail!("unsupported database URL scheme: {url}") }
}

pub fn expand_env_vars(url: &str) -> String {
    // Replace ${VAR_NAME} with env var value
    // Return original token unexpanded if var is missing (don't panic)
}

pub async fn init_pool(url: &str) -> anyhow::Result<AnyPool> {
    let expanded = expand_env_vars(url);
    let masked   = mask_password(&expanded); // for logging only
    log::info!("connecting to database: {masked}");
    let pool = AnyPool::connect(&expanded).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}
```

---

## Schema — portable across all three drivers

Use SQL that works on SQLite, PostgreSQL, and MySQL without dialect
switching. Key constraints:

- No `SERIAL` or `AUTOINCREMENT` for PKs — use TEXT UUID (works everywhere)
- No `BOOLEAN` type — use `INTEGER` (0/1) for SQLite compat
- No `RETURNING` clause — not supported in MySQL
- Use `TEXT` for timestamps (RFC 3339 strings) — avoids timezone dialect hell
- `IF NOT EXISTS` on all CREATE TABLE statements

```sql
-- scan_events: one row per completed scan
CREATE TABLE IF NOT EXISTS scan_events (
    id            TEXT    NOT NULL PRIMARY KEY,
    scanned_at    TEXT    NOT NULL,
    target_host   TEXT    NOT NULL,
    target_port   INTEGER NOT NULL,
    service       TEXT    NOT NULL,
    status        TEXT    NOT NULL,
    event_json    TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_scan_events_host
    ON scan_events (target_host, target_port);

CREATE INDEX IF NOT EXISTS idx_scan_events_scanned_at
    ON scan_events (scanned_at);

-- schedules: active periodic scan jobs
CREATE TABLE IF NOT EXISTS schedules (
    id            TEXT    NOT NULL PRIMARY KEY,
    label         TEXT,
    targets_file  TEXT    NOT NULL,
    cron_expr     TEXT    NOT NULL,
    standards     TEXT    NOT NULL,
    last_run_at   TEXT,
    next_run_at   TEXT    NOT NULL,
    enabled       INTEGER NOT NULL DEFAULT 1
);

-- targets: named target groups
CREATE TABLE IF NOT EXISTS targets (
    id            TEXT    NOT NULL PRIMARY KEY,
    label         TEXT    NOT NULL,
    hosts_json    TEXT    NOT NULL
);
```

---

## Migration strategy

Use sqlx migrations (`sqlx migrate`) with numbered files:

```
migrations/
  0001_initial.sql       ← schema above
  0002_add_cert_expiry.sql  ← future: add cert_expiry column for fast queries
```

sqlx tracks applied migrations in a `_sqlx_migrations` table automatically.
This works identically on SQLite, PostgreSQL, and MySQL.

---

## Useful queries for OSS history/trend features

```sql
-- Last scan for a specific host
SELECT event_json FROM scan_events
WHERE target_host = ? AND target_port = ?
ORDER BY scanned_at DESC LIMIT 1;

-- All scans for a host in the last 30 days
SELECT scanned_at, status, event_json FROM scan_events
WHERE target_host = ?
  AND scanned_at >= datetime('now', '-30 days')
ORDER BY scanned_at DESC;

-- Hosts with at least one FAIL in last 24h
SELECT DISTINCT target_host, target_port FROM scan_events
WHERE status IN ('fail', 'warn')
  AND scanned_at >= datetime('now', '-1 day');

-- Compliance score trend for a host (last 10 scans)
-- Extract from event_json in application layer — don't store scores separately
SELECT scanned_at, event_json FROM scan_events
WHERE target_host = ?
ORDER BY scanned_at DESC LIMIT 10;
```

---

## What stays the same

- Table schema is identical across drivers — no driver-specific columns
- `event_json` stores the full QEM ScanEvent — single source of truth
- Compliance scores are never stored separately — always derived from event_json
  at read time (keeps schema stable as compliance rules evolve)
- SQLite WAL mode enabled at startup for SQLite URLs only

## What changes from current state.rs

- Replace `rusqlite::Connection` with `sqlx::AnyPool`
- Add `expand_env_vars()` for ${VAR} substitution in URL
- Add `mask_password()` for log safety
- Add `detect_driver()` for URL scheme routing
- Remove `rusqlite` from Cargo.toml dependencies
- Add sqlx migration files under `migrations/`

---

## OSS scope freeze — database features included

| Feature                          | OSS | Enterprise |
|----------------------------------|-----|------------|
| SQLite (default, zero config)    | ✅  |            |
| PostgreSQL (--features postgres) | ✅  |            |
| MySQL/MariaDB (--features mysql) | ✅  |            |
| Local scan history + trend       | ✅  |            |
| Scheduled job persistence        | ✅  |            |
| Single-node only                 | ✅  |            |
| Multi-node / shared DB fleet     |     | 🔒         |
| Cross-node query / aggregation   |     | 🔒         |
| DB connection pooling >5         |     | 🔒         |
| Read replicas / HA config        |     | 🔒         |

Note: PostgreSQL in OSS is single-node (one qxscan instance → one DB).
The moment two qxscan nodes share a PostgreSQL database for fleet
coordination, that is an Enterprise feature.

