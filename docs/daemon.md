# Daemon & State Store

> **Background service for scheduled scanning with SQLite persistence.**

---

## Daemon Overview

The QXScan daemon (`qxscan server`) runs as a background process that
executes scan jobs on a schedule and stores results in SQLite. It has no
UI, no users, and no authentication — it is a job runner.

### Startup Sequence

```
1. Load qxscan.toml
2. Check PID file — abort if daemon is already running
3. Initialize SQLite database (create + migrate)
4. Bind HTTP server on config.server.bind
   → /status  (health check + recent scan summary)
   → /metrics (Prometheus exposition)
5. Write PID file
6. Spawn schedule runner loop
7. SIGTERM → graceful shutdown → remove PID file
```

### CLI

```bash
# Start the daemon
qxscan server start

# Check status
qxscan server status

# Stop the daemon
qxscan server stop

# Restart
qxscan server restart
```

### Status Output

```
daemon: running (PID 12345)
pid file: /home/user/.qxscan/qxscan.pid
state db: sqlite:///home/user/.qxscan/state.db
bind: 127.0.0.1:9412

recent scans:
  a1b2c3d4 → example.com:443 (Pass)

schedules (2):
  e5f6 Daily scan (daily) next: 2026-06-28 02:00 UTC
```

---

## Daemon Architecture

```
┌──────────────────────────────────────────────┐
│                 qxscan daemon                 │
│                                              │
│  ┌──────────────┐     ┌──────────────────┐   │
│  │  HTTP Server  │     │  Schedule Runner │   │
│  │  /status      │     │  every 60s       │   │
│  │  /metrics     │     │  check + spawn   │   │
│  └──────┬───────┘     └────────┬─────────┘   │
│         │                      │             │
│         └──────────┬───────────┘             │
│                    ▼                         │
│          ┌──────────────────┐                │
│          │  SQLite (state)  │                │
│          │  scan_events     │                │
│          │  scan_reports    │                │
│          │  schedules       │                │
│          └──────────────────┘                │
└──────────────────────────────────────────────┘
```

---

## Scheduler

### Schedule Runner Loop

```
every 60 seconds:
  SELECT schedules WHERE next_run_at <= now() AND enabled = 1
  for each due schedule:
    tokio::spawn(run_scan(schedule))
    UPDATE next_run_at = next_cron_tick(cron_expr)
  sleep(60s)
```

### Named Intervals

| Interval | Cron Expression | UTC Time |
|----------|----------------|----------|
| `hourly` | `0 * * * *` | Top of hour |
| `daily` | `0 2 * * *` | 02:00 daily |
| `weekly` | `0 2 * * 1` | Monday 02:00 |
| `monthly` | `0 2 1 * *` | 1st of month 02:00 |

Custom cron expressions are also supported (e.g. `*/30 * * * *` for every
30 minutes).

### Schedule Management

```bash
# Add a daily scan
qxscan schedule add \
  --targets-file /etc/qxscan/targets.txt \
  --interval daily \
  --standards pci-dss,hipaa

# Add with custom cron
qxscan schedule add \
  --targets-file /etc/qxscan/targets.txt \
  --cron "0 */6 * * *" \
  --standards pci-dss,soc2 \
  --label "6-hourly PCI+SOC2"

# List schedules
qxscan schedule list

# Remove a schedule
qxscan schedule remove <schedule-id>

# Preview future runs
qxscan schedule preview --count 10
```

---

## State Store (SQLite)

QXScan uses SQLite for local persistence with zero configuration. The
database file is created automatically on first run.

### Configuration

```toml
[database]
url = "sqlite://~/.qxscan/state.db"
max_connections = 5
connect_timeout_s = 10
```

The `~` is expanded to the home directory. Environment variables in the
URL are also expanded:

```toml
url = "postgres://${DB_USER}:${DB_PASS}@localhost/qxscan"
```

### Database Schema

#### `scan_events` — Scan history

```sql
CREATE TABLE IF NOT EXISTS scan_events (
    id            TEXT    NOT NULL PRIMARY KEY,
    scanned_at    TEXT    NOT NULL,       -- RFC 3339
    target_host   TEXT    NOT NULL,
    target_port   INTEGER NOT NULL,
    service       TEXT    NOT NULL,
    status        TEXT    NOT NULL,       -- pass|fail|warn|error|timeout
    event_json    TEXT    NOT NULL        -- Full QEM ScanEvent JSON
);
```

#### `scan_reports` — Rendered report storage

```sql
CREATE TABLE IF NOT EXISTS scan_reports (
    id            TEXT    NOT NULL PRIMARY KEY,
    scan_id       TEXT    NOT NULL,
    format        TEXT    NOT NULL,       -- json|html|terminal
    content       TEXT    NOT NULL,       -- Rendered report content
    file_path     TEXT,                   -- Original file path (if any)
    created_at    TEXT    NOT NULL        -- RFC 3339
);
```

#### `schedules` — Periodic scan definitions

```sql
CREATE TABLE IF NOT EXISTS schedules (
    id            TEXT    NOT NULL PRIMARY KEY,
    label         TEXT,
    targets_file  TEXT    NOT NULL,
    cron_expr     TEXT    NOT NULL,
    standards     TEXT    NOT NULL,       -- Comma-separated slugs
    last_run_at   TEXT,                   -- RFC 3339 or NULL
    next_run_at   TEXT    NOT NULL,
    enabled       INTEGER NOT NULL DEFAULT 1
);
```

#### `targets` — Target groups

```sql
CREATE TABLE IF NOT EXISTS targets (
    id            TEXT    NOT NULL PRIMARY KEY,
    label         TEXT    NOT NULL,
    hosts_json    TEXT    NOT NULL        -- JSON array of hosts
);
```

### Portability Rules

The schema is designed to work across SQLite, PostgreSQL, and MySQL:

- **No `SERIAL` / `AUTOINCREMENT`** — use TEXT UUID (works everywhere)
- **No `BOOLEAN`** — use INTEGER 0/1 (SQLite compatible)
- **No `RETURNING`** — not supported in MySQL
- **Timestamps as TEXT RFC 3339** — avoids timezone dialect differences
- **`IF NOT EXISTS`** — on all CREATE TABLE / INDEX statements

### Migrations

SQLx tracks applied migrations in the `_sqlx_migrations` table, applied
in version order:

```
migrations/
├── 0001_initial.sql          ← scan_events, schedules, targets
└── 0002_scan_reports.sql     ← scan_reports table
```

### Implementation Details

- **URL expansion**: `${ENV_VAR}` and `$ENV_VAR` patterns are expanded at startup
- **Password masking**: Connection URLs are masked in logs (`user:***@host`)
- **Auto-creation**: Database directory and file are created automatically
- **WAL mode**: SQLite journal mode is set to WAL for concurrent read performance
- **Best-effort**: Scan succeeds even if database is unavailable

---

## HTTP Endpoints

When the daemon is running, it exposes two HTTP endpoints:

### `GET /status`

Health check returning JSON with recent scan and schedule summary:

```json
{
  "uptime_secs": 12345,
  "scans_count": 42,
  "schedules_count": 3
}
```

### `GET /metrics`

Prometheus-compatible metrics:

```
# HELP qxscan_uptime_seconds Daemon uptime
# TYPE qxscan_uptime_seconds gauge
qxscan_uptime_seconds 1
# HELP qxscan_scans_total Total scans completed
# TYPE qxscan_scans_total counter
qxscan_scans_total 42
```
