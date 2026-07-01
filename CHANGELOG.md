# Changelog

All notable changes to QXScan are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] — 2026-06-30

### Added

- Core scanner engine with TLS probe (handshake, cert, cipher, PQC detection)
- Compliance engine supporting PCI-DSS 4.0, HIPAA, SOC 2, FISMA, and PQC readiness
- CLI commands: `scan`, `server`, `schedule`, `export`, `report`, `version`
- Report sub-commands: `render`, `list`, `show`, `remove`
- Export formats: QEM, Prometheus, OCSF, CEF
- Report formats: terminal, JSON, HTML
- Daemon mode with scheduled scanning (named intervals + custom cron)
- SQLite state store (scan history, schedules, targets, scan_reports)
- Documentation suite under `docs/`: getting-started, architecture, QEM spec,
  compliance, export, daemon, testing, contributing, enterprise
- `README.md` with badges, CLI overview, and links to docs
- `CONTRIBUTING.md` — redirects to full contributing guide in docs
- `SECURITY.md` — vulnerability reporting policy
- `CHANGELOG.md` — changelog tracking
- Docker demo infrastructure (PQC fixtures, legacy TLS, ALB, SMTP)
- 66 unit + integration tests across scanner, TLS, compliance, and report modules
- 27-test demo suite (`demo_suite.sh`) + content assertion suite (`demo_suite_v2.sh`)
- CI workflow (build, test, clippy, fmt)
- Multi-arch release workflow (linux/musl, macOS arm64/x86_64)
- Windows cross-compile targets (`x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`)
- **Scanner provenance (QEM)**: `scanner` block in `ScanEvent` with `name`, `version`,
  `engine`, and `platform` fields for enterprise metadata
- `ROADMAP.md` — v0.1 completed, v0.2/v0.3 planned, Enterprise, non-goals
- `CODE_OF_CONDUCT.md` — Contributor Covenant 2.1
- `docs/protocols.md` — All 8 supported protocols with port mappings and TLS behaviour
- Architecture diagram in `README.md`
- "Supported Versions" section in `SECURITY.md`
- `docs/testing.md` — Updated to reflect current test scripts (test_docker.sh,
  classification_test.sh, test_all.sh)
- `docs/qem-spec.md` — Documented `ScannerInfo` in ScanEvent envelope
- `docs/contributing.md` — Linked to CODE_OF_CONDUCT.md, updated test references
- `docs/README.md` — Added protocols.md to documentation index
- `docs/architecture.md` — Updated source tree to mention ScannerInfo
- `docs/enterprise.md` — Added ROADMAP.md reference

### Changed

- **License**: Apache 2.0
- Upgraded `sqlx` from 0.7.4 to 0.8.6 — fixes 4 of 5 `cargo audit` vulnerabilities
- Switched from `sqlx::AnyPool` to `sqlx::SqlitePool` — eliminates LTO driver
  registration panic
- Database auto-creation: `create_dir_all` + `create_if_missing(true)` ensures
  zero-config setup
- CLI restructured: `qxscan report` now uses subcommands
- File header comments added to all Rust source files

### Fixed

- **PQC-1.1 scoring bug**: `pqc_hybrid == false` now correctly returns `warn`
  (was incorrectly returning `pass`)
- Pass findings now have `remediation: None` (was non-null for some pass findings)
- All clippy warnings resolved (dead_code, unused_imports)
- Demo suite v2 exit-code assertion bug fixed
- **NoTls classification**: Plain HTTP services now correctly detected as
  `ℹ️ No TLS` instead of `❌ Error`. Compliance findings are still generated
  — missing TLS triggers PCI-DSS/HIPAA/SOC2/FISMA/PQC fail findings.
- **Unreachable host timeout**: TCP connection errors (timeout, refused)
  are now correctly classified as `⏰ Timeout` or `❌ Connection Failed`
  instead of `❌ Error`. Underlying `io::Error` was not being detected
  through anyhow's error chain (`e.source()` returns `None` for root causes).
- **Docker CA bundle**: Added `ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt`
  and `update-ca-certificates --fresh` so vendored OpenSSL finds the
  system CA store inside the container.
- **Windows cross-compile targets**: Added `x86_64-pc-windows-msvc` and
  `aarch64-pc-windows-msvc` to the release workflow with platform-aware
  packaging (.zip) and checksums (PowerShell `Get-FileHash`).
- **Release workflow awk escaping**: Fixed double-escape of `\[` in release
  notes extraction that caused empty release body.

### Removed

- Vestigial `[features]` section from `Cargo.toml`
- Dead code: `detect_driver()`, `pid_path()`, unused imports
