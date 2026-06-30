-- 0001_initial.sql — QXScan OSS portable schema
-- Compatible with SQLite, PostgreSQL, and MySQL/MariaDB.
-- No SERIAL/AUTOINCREMENT, no BOOLEAN, no RETURNING, TEXT timestamps.

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

CREATE TABLE IF NOT EXISTS targets (
    id            TEXT    NOT NULL PRIMARY KEY,
    label         TEXT    NOT NULL,
    hosts_json    TEXT    NOT NULL
);
