# QXScan

<p align="center">
  <img src="https://img.shields.io/badge/license-Apache%202.0-blue" alt="License">
  <img src="https://img.shields.io/badge/rust-1.75%2B-orange" alt="Rust">
  <img src="https://img.shields.io/badge/tests-66%20passing-brightgreen" alt="Tests">
  <img src="https://img.shields.io/badge/binary-%3C10%20MB-lightgrey" alt="Binary Size">
</p>

> **The fastest, simplest, self-contained on-prem security and compliance
> scanner that runs as a single binary.**

QXScan connects to servers, collects TLS evidence, evaluates compliance
against regulatory standards (PCI-DSS, HIPAA, SOC2, FISMA, PQC), and
produces structured output (QEM) that any downstream tool can consume.

---

## Quick Start

```bash
# Scan a single target
qxscan scan example.com

# Scan with compliance standards
qxscan scan example.com --standards pci-dss,hipaa,soc2

# Scan and output JSON
qxscan scan example.com --standards pci-dss --output json --report-file report.json

# Start the daemon for scheduled scanning
qxscan server start
```

## Install

```bash
cargo build --release
# Binary at ./target/release/qxscan
```

Or use Docker:

```bash
docker compose build scanner
docker compose run --rm scanner scan example.com
```

---

## Documentation

All documentation is in the [`docs/`](docs/) directory. Start with the
[documentation index](docs/README.md) for a full overview of what's
available.

| Document | Description |
|----------|-------------|
| [Getting Started](docs/getting-started.md) | Installation, quick start, full CLI reference |
| [Architecture](docs/architecture.md) | Pipeline overview, source tree, module contracts |
| [QEM Spec](docs/qem-spec.md) | QX Event Model — open, versioned scan result format |
| [Compliance](docs/compliance.md) | Standards, controls, scoring algorithm |
| [Export](docs/export.md) | Output formats — Prometheus, OCSF, CEF |
| [Daemon](docs/daemon.md) | Background service, scheduler, SQLite state store |
| [Testing](docs/testing.md) | Build, test, lint, demo suite, CI |
| [Contributing](docs/contributing.md) | Coding conventions, PR checklist |
| [Enterprise](docs/enterprise.md) | OSS vs Enterprise feature boundary, upgrade path |

---

## CLI Overview

```
qxscan [OPTIONS] <COMMAND>

Commands:
  scan      Scan targets for TLS posture and compliance
  server    Manage the qxscan daemon (start | stop | restart | status)
  schedule  Schedule periodic scans
  export    Export a scan report to an observability format
  report    Render a stored report or list stored reports
  version   Print version and build metadata
```

---

## Supported Standards

| Standard | Controls |
|----------|----------|
| PCI-DSS 4.0 | TLS version, cipher strength, forward secrecy, certificate validity |
| HIPAA Security Rule | TLS version, cipher strength, forward secrecy, certificate validity |
| SOC 2 Type II | TLS version, cipher strength, forward secrecy, certificate validity |
| FISMA / NIST SP 800-52r2 | TLS version, cipher strength, forward secrecy, certificate validity |
| PQC Readiness (CNSA 2.0) | PQC hybrid detection, TLS 1.3, combined readiness |

---

## License

Apache 2.0 — see [LICENSE](./LICENSE).

QX Event Model (QEM) is permanently open source.

---

**Security:** See [SECURITY.md](./SECURITY.md) for vulnerability reporting instructions.
