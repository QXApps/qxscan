# Getting Started with QXScan

> **QXScan** — The fastest, simplest, self-contained on-prem security and
> compliance scanner that runs as a single binary.

---

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/QXApps/qxscan.git
cd qxscan

# Build release binary
cargo build --release

# Verify
./target/release/qxscan --version
```

> **Note:** If scanning returns `❌ Error` with TLS certificate verification
> failures, set the `SSL_CERT_FILE` environment variable to your system's CA
> bundle before running:
> ```bash
> export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
> ```
> This is required on systems where vendored OpenSSL cannot auto-detect the
> system CA store. The Docker image handles this automatically.

### From Docker

```bash
docker pull ghcr.io/qxapps/qxscan:latest
# or build locally
docker compose build scanner
```

### Requirements

- **Rust** 1.75+ (if building from source)
- **No runtime dependencies** — the binary is statically linked
- **No database setup** — SQLite is bundled, zero config

---

## Quick Start

### Scan a single target

```bash
# Basic TLS posture scan
qxscan scan example.com

# Scan with compliance standards
qxscan scan example.com --standards pci-dss,hipaa,soc2

# Scan with JSON output
qxscan scan example.com --standards pci-dss --output json --out report.json

# Scan a non-standard port
qxscan scan db.internal --port 5433 --service postgres
```

### Scan multiple targets

```bash
# Targets file (one per line)
cat > targets.txt <<EOF
server01.corp.com
server02.corp.com:9443
10.1.0.0/24
EOF

qxscan scan --targets-file targets.txt --parallel 16
```

### View scan reports

```bash
# List stored reports
qxscan report list

# Show a specific report
qxscan report show <report-id>

# Save a report to a file
qxscan report show <report-id> --out report.html
```

---

## CLI Reference

### Top-level

```
qxscan [OPTIONS] <COMMAND>

Options:
  -v, --verbose    Enable verbose debug output
  -q, --quiet      Suppress non-error output
  -h, --help       Print help
  -V, --version    Print version

Commands:
  scan      Scan targets for TLS posture and compliance
  server    Manage the qxscan daemon (start | stop | restart | status)
  schedule  Schedule periodic scans
  export    Export a scan report to an observability format
  report    Render a stored report or list stored reports
  version   Print version and build metadata
```

### `qxscan scan`

```bash
qxscan scan [TARGET] [OPTIONS]

Target (positional, conflicts with --targets-file):
  [TARGET]               hostname, IP, or host:port

Options:
      --port PORT            Port override [default: 443]
      --service TYPE         Service type preset [default: https]
                             (https, smtp, imap, pop3, postgres, mysql, ldap, ftp)
      --standards LIST       Compliance standards (comma-separated)
                             [default: pci-dss] (pci-dss, hipaa, soc2, fisma, pqc)
      --output FORMAT        Output format [default: terminal]
                             (terminal, json, html)
      --report-file PATH     Write report to file (required for json/html output)
      --timeout SECS         Connection timeout [default: 10]
      --no-verify            Skip TLS certificate verification
      --concurrency N        Concurrent scan workers [default: 4]
      --targets-file FILE    File containing one target per line

Examples:
  qxscan scan server01
  qxscan scan server01 --standards pci-dss,hipaa --output json --report-file report.json
  qxscan scan --targets-file subnet.txt --parallel 32 --standards pci-dss
  qxscan scan db.internal --service postgres --port 5433
```

### `qxscan server`

```bash
qxscan server <COMMAND>

Commands:
  start     Start the daemon
  stop      Stop the daemon
  restart   Restart the daemon
  status    Show daemon status and recent activity
```

### `qxscan schedule`

```bash
qxscan schedule <COMMAND> [OPTIONS]

Commands:
  add       Add a new schedule
  list      List active schedules
  remove    Remove a schedule by ID
  preview   Show next N scheduled runs

Schedule add options:
      --targets-file FILE    Targets file (required)
      --interval INTERVAL    Named interval: hourly|daily|weekly|monthly
      --cron EXPR            Cron expression (e.g. "0 2 * * *")
      --standards LIST       Compliance standards (comma-separated)
      --label TEXT           Human-readable label
```

### `qxscan export`

```bash
qxscan export --from FILE --format FORMAT [--out FILE]

Formats:
  qem         Canonical QEM JSON (versioned, passthrough)
  prometheus  Prometheus text exposition format
  ocsf        OCSF class 2004 — Vulnerability Finding
  cef         Common Event Format (syslog / SIEM)
```

### `qxscan report`

```bash
qxscan report <COMMAND> [OPTIONS]

Commands:
  render    Render a scan report from a QEM file
  list      List stored reports from the database
  show      Show the full content of a stored report by ID
  remove    Remove a stored report by ID

Render options:
      --from FILE    Input report file (QEM JSON)
      --format FMT   Output format: terminal|json|html [default: terminal]
      --out FILE     Output file path

List options:
      --scan-id ID   Filter by scan ID
      --format FMT   Filter by format (json, html)
      --limit N      Maximum reports [default: 20]

Show options:
      <ID>           Report ID (full UUID or prefix from list)
      --out FILE     Output file path

Remove options:
      <ID>           Report ID to remove
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success, all controls passed |
| 1 | General error (config, runtime, I/O) |
| 2 | Unrecognised flag or argument |
| 3 | Scan complete, at least one FAIL finding |
| 4 | Target unreachable / timeout |

---

## Configuration

QXScan uses a `qxscan.toml` file for runtime configuration:

```toml
[database]
url = "sqlite://~/.qxscan/state.db"
max_connections = 5
connect_timeout_s = 10

[scan]
timeout_s = 10
concurrency = 4
standards = ["pci-dss"]

[server]
bind = "127.0.0.1:9412"
```

Environment variables in config values are expanded at startup:
```toml
url = "postgres://${DB_USER}:${DB_PASS}@localhost/qxscan"
```
