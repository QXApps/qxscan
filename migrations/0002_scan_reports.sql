-- 0002_scan_reports.sql — Rendered report storage
-- Compatible with SQLite, PostgreSQL, and MySQL/MariaDB.
-- Allows storing rendered HTML/JSON/terminal reports alongside scan events
-- for later retrieval and export.

CREATE TABLE IF NOT EXISTS scan_reports (
    id            TEXT    NOT NULL PRIMARY KEY,
    scan_id       TEXT    NOT NULL,
    format        TEXT    NOT NULL,
    content       TEXT    NOT NULL,
    file_path     TEXT,
    created_at    TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_scan_reports_scan_id
    ON scan_reports (scan_id);

CREATE INDEX IF NOT EXISTS idx_scan_reports_format
    ON scan_reports (format);
