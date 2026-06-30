# QXScan Architecture

> **Pipeline:** `Scan → Evidence → QEM → Compliance → Export`

---

## Overview

QXScan is built as a **linear pipeline** where each stage produces output
that feeds into the next. Every module in the codebase maps to exactly one
stage. No module spans two stages.

```
┌─────────────────────────────────────────────────────────┐
│  CLI (clap)  src/cli/                                   │
│  scan | server | schedule | export | report | version   │
└──────────────────────┬──────────────────────────────────┘
                       │
          ┌────────────┼─────────────┐
          ▼            ▼             ▼
    ┌──────────┐  ┌─────────┐  ┌──────────────┐
    │ scanner/ │  │  tls/   │  │ compliance/  │
    │ service  │  │handshake│  │  scoring     │
    │ (probes) │  │  cert   │  │  standards/  │
    │          │  │ cipher  │  │              │
    │          │  │  pqc    │  │              │
    └────┬─────┘  └────┬────┘  └──────┬───────┘
         │              │              │
         └──────────────▼──────────────┘
                        │ produces
                        ▼
              ┌─────────────────┐
              │  qem/ (QEM)   │  ← STABLE PUBLIC SCHEMA
              │  metadata.rs    │    ScanEvent envelope
              │  observation.rs │    TlsInfo, CertInfo (Evidence)
              │  finding.rs     │    Finding, Severity
              │  compliance.rs  │    ComplianceScore
              └────────┬────────┘
                       │ consumed by
          ┌────────────┼────────────────┐
          ▼            ▼                ▼
    ┌──────────┐  ┌─────────┐  ┌───────────────┐
    │ report/  │  │ export/ │  │   server/     │
    │ terminal │  │  qem    │  │ state (SQLite)│
    │ json     │  │  ocsf   │  │   pid         │
    │ html     │  │  cef    │  └──────┬────────┘
    └──────────┘  │ prometheus│        │
                  └─────────┘        ▼
                               ┌──────────────┐
                               │  schedule/   │
                               │  cron        │
                               │  runner      │
                               └──────────────┘
```

---

## Data Flow

### Single scan run

```
CLI args
  → resolve targets          (scanner/service.rs)
  → tls::handshake           (TCP connect + TLS negotiate)
  → tls::cert                (parse chain → CertInfo)
  → tls::cipher              (cipher suite inspection)
  → tls::pqc                 (PQC hybrid key exchange detection)
  → Evidence assembled       (observation.rs: TlsInfo + CertInfo)
  → compliance::scoring::evaluate() per requested standard
      → standard::evaluate(&event) → Vec<Finding>
      → ComplianceScore::from_findings()
  → ScanEvent (QEM) assembled: metadata + observation + findings + compliance
  → report::* renderer       (terminal / JSON / HTML)
  → export::* sink           (QEM / OCSF / CEF / Prometheus)
  → server::state::insert()  (persist to SQLite if daemon mode)
```

### Daemon scan loop

```
Every 60 seconds:
  SELECT schedules WHERE next_run_at <= now() AND enabled = 1
  for each due schedule:
    tokio::spawn(run_scan(schedule))
    UPDATE next_run_at = next_cron_tick(cron_expr)
  sleep(60s)
```

---

## Source Tree

```
qxscan/
├── Cargo.toml
├── Cargo.lock
├── qxscan.toml                  ← Runtime configuration
├── Dockerfile
├── docker-compose.yml
├── Makefile
│
├── src/
│   ├── main.rs                  ← Thin router (< 20 lines)
│   │
│   ├── cli/                     ← CLAP definitions + dispatch only
│   │   ├── mod.rs               ← Cli struct, Commands enum, dispatch()
│   │   ├── scan.rs              ← ScanArgs + run()
│   │   ├── server.rs            ← ServerArgs + run()
│   │   ├── schedule.rs          ← ScheduleArgs + run()
│   │   ├── export.rs            ← ExportArgs + run()
│   │   ├── report.rs            ← ReportArgs + run() — render, list, show, remove
│   │   └── version.rs           ← VersionArgs + run()
│   │
│   ├── qem/                   ← QEM: QX Event Model (stable schema)
│   │   ├── mod.rs               ← Re-exports all public types
│   │   ├── metadata.rs          ← ScanEvent envelope, TargetInfo, ScanStatus
│   │   ├── observation.rs       ← Evidence: TlsInfo, CertInfo (raw wire data)
│   │   ├── finding.rs           ← Finding, FindingStatus, Severity
│   │   └── compliance.rs        ← ComplianceScore, grade_from_score()
│   │
│   ├── scanner/                 ← Pure probe logic (no I/O side effects)
│   │   ├── mod.rs
│   │   └── service.rs           ← ServiceType enum + default ports
│   │
│   ├── tls/                     ← TLS probe implementations
│   │   ├── mod.rs
│   │   ├── handshake.rs         ← TCP connect + TLS negotiation
│   │   ├── cert.rs              ← Certificate chain → CertInfo
│   │   ├── cipher.rs            ← Cipher suite inspection
│   │   └── pqc.rs               ← PQC hybrid key exchange detection
│   │
│   ├── compliance/              ← Policy engine (pure functions)
│   │   ├── mod.rs
│   │   ├── scoring.rs           ← evaluate(event, standard) dispatch
│   │   └── standards/
│   │       ├── mod.rs
│   │       ├── pci_dss.rs       ← PCI-DSS 4.0 controls
│   │       ├── hipaa.rs         ← HIPAA Security Rule
│   │       ├── soc2.rs          ← SOC 2 Type II
│   │       ├── fisma.rs         ← FISMA / NIST SP 800-52r2
│   │       └── pqc.rs           ← CNSA 2.0 / NIST PQC readiness
│   │
│   ├── export/                  ← Exporter trait + OSS format writers
│   │   ├── mod.rs               ← pub trait Exporter
│   │   ├── qem.rs               ← Canonical QEM JSON (passthrough)
│   │   ├── prometheus.rs        ← Prometheus text exposition
│   │   ├── ocsf.rs              ← OCSF class 2004
│   │   └── cef.rs               ← CEF / syslog
│   │
│   ├── report/                  ← Human-readable renderers
│   │   ├── mod.rs
│   │   ├── terminal.rs          ← Coloured TTY output
│   │   ├── json.rs              ← Pretty-printed QEM JSON
│   │   └── html.rs              ← Self-contained HTML report
│   │
│   ├── server/                  ← Daemon lifecycle
│   │   ├── mod.rs               ← Config, start/stop/restart/status
│   │   ├── pid.rs               ← PID file management
│   │   └── state.rs             ← SQLite: init, CRUD, history, reports
│   │
│   └── schedule/                ← Periodic scan scheduler
│       ├── mod.rs
│       ├── cron.rs              ← Named intervals → cron expressions
│       └── runner.rs            ← Async tokio scan loop
│
├── migrations/
│   ├── 0001_initial.sql         ← Tables: scan_events, schedules, targets
│   └── 0002_scan_reports.sql    ← Table: scan_reports (rendered report storage)
│
└── demo/
    ├── demo_suite.sh            ← 27-test process-level suite
    ├── demo_suite_v2.sh         ← Content assertion suite (jq)
    ├── targets.txt
    ├── caddy-pqc/               ← Canonical "PQC-ready" fixture
    ├── certs/generate.sh        ← Test cert generation
    └── ...
```

---

## Module Contracts

### `src/main.rs`
- Under 20 lines. Only imports: `cli`, `clap::Parser`. No business logic.

### `src/cli/*`
- Only module allowed to import `clap`.
- Handlers return `anyhow::Result<()>`, delegate to library modules.
- No direct TLS, SQLite, or filesystem access.

### `src/qem/*` (QEM)
- Zero dependencies on other qxscan modules.
- External deps only: `serde`, `chrono`, `uuid`, `std`.
- Every public struct field must be `Serialize + Deserialize`.

### `src/scanner/*` and `src/tls/*`
- Pure functions. No global state, no file I/O, no `println!`.
- Unit-testable without a network connection.

### `src/compliance/*`
- Pure functions. Input: `&ScanEvent`. Output: `Vec<Finding>`.
- No network, no file I/O, no randomness.
- Each standard exports one public function: `evaluate()`.
- `scoring.rs` is the single dispatch point.

### `src/export/*`
- All exporters implement the `Exporter` trait.
- Read-only consumers of `ScanEvent`.

### `src/server/*`
- `pid.rs`: honours `QXSCAN_PID_FILE`, falls back to `~/.qxscan/qxscan.pid`.
- `state.rs`: honours `QXSCAN_STATE_FILE`, falls back to `~/.qxscan/state.db`.

### `src/schedule/*`
- `cron.rs` owns all named-interval → cron-expression mappings.
- `runner.rs` calls `compliance::scoring::evaluate()` — never standards directly.

---

## Key Design Decisions

1. **Single binary** — Statically linked, zero runtime dependencies. SQLite is bundled.
2. **Pure functions** — Scanner, TLS, and compliance modules have no side effects.
3. **QEM as stable schema** — The event model is the public API contract.
4. **SQLite by default** — Zero-config persistence. No external database required.
5. **Best-effort persistence** — Scans succeed even if the database is unavailable.
6. **No async in critical path** — TLS handshakes use blocking I/O in worker threads.
