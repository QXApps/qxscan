# Export Layer

> **Export QEM scan results to open observability formats.**

---

## Overview

The export layer converts `ScanEvent` (QEM) documents into various open
formats. All exporters implement the `Exporter` trait and are read-only
consumers of scan results.

```
ScanEvent → Exporter trait → QEM | Prometheus | OCSF | CEF
```

---

## OSS Export Formats

| Format | Module | Description | Use Case |
|--------|--------|-------------|----------|
| `qem` | `export/qem.rs` | Canonical QEM JSON (passthrough) | Archival, pipeline ingestion |
| `prometheus` | `export/prometheus.rs` | Prometheus text exposition | `/metrics` endpoint, monitoring |
| `ocsf` | `export/ocsf.rs` | OCSF class 2004 | SIEM integration |
| `cef` | `export/cef.rs` | Common Event Format | Syslog, legacy SIEM |

---

## Exporter Trait

```rust
// src/export/mod.rs
pub trait Exporter {
    fn export(&self, events: &[ScanEvent]) -> anyhow::Result<()>;
}
```

The trait is public and documented. Third-party tools can implement custom
exporters in external crates. Enterprise exporters (Elastic, Splunk,
Datadog, OTLP) implement this trait outside this repository.

---

## CLI Usage

```bash
# Export a QEM file to Prometheus format
qxscan export --from scan_result.json --format prometheus

# Export to OCSF and save to file
qxscan export --from scan_result.json --format ocsf --out findings.ocsf.json

# Export to CEF for syslog ingestion
qxscan export --from scan_result.json --format cef
```

### Arguments

| Argument | Description |
|----------|-------------|
| `--from FILE` | Input QEM JSON file (required) |
| `--format FMT` | Output format: `qem`, `prometheus`, `ocsf`, `cef` |
| `--out FILE` | Output file (defaults to stdout) |

**Note:** OSS export is single-shot — stdout or file. Continuous push,
streaming, retry, batching, and deduplication are Enterprise features.

---

## QEM Format (Passthrough)

The `qem` exporter produces canonical QEM JSON. This is a versioned
passthrough format — the input is validated and re-serialized to ensure
schema compliance.

```json
{
  "schema_version": "1",
  "scan_id": "a1b2c3d4-...",
  "target": { "host": "example.com", "port": 443 },
  "overall_status": "pass",
  "tls": { ... },
  "findings": [ ... ],
  "compliance": { ... }
}
```

---

## Prometheus Format

The `prometheus` exporter produces OpenMetrics-compatible text exposition.

### Metric Names (locked)

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `qxscan_scan_duration_ms` | gauge | `host`, `port` | Scan duration in milliseconds |
| `qxscan_compliance_score` | gauge | `host`, `port`, `standard` | Compliance score (0–100) |
| `qxscan_finding_total` | counter | `host`, `port`, `standard`, `status` | Finding count by status |
| `qxscan_cert_days_to_expiry` | gauge | `host`, `port` | Days until certificate expiry |

### Example Output

```
# HELP qxscan_scan_duration_ms Scan duration in milliseconds
# TYPE qxscan_scan_duration_ms gauge
qxscan_scan_duration_ms{host="example.com",port="443"} 57

# HELP qxscan_compliance_score Compliance score (0-100)
# TYPE qxscan_compliance_score gauge
qxscan_compliance_score{host="example.com",port="443",standard="pci_dss"} 100
qxscan_compliance_score{host="example.com",port="443",standard="hipaa"} 75

# HELP qxscan_finding_total Finding count by status
# TYPE qxscan_finding_total counter
qxscan_finding_total{host="example.com",port="443",standard="pci_dss",status="pass"} 4
qxscan_finding_total{host="example.com",port="443",standard="pci_dss",status="fail"} 0

# HELP qxscan_cert_days_to_expiry Days until certificate expiry
# TYPE qxscan_cert_days_to_expiry gauge
qxscan_cert_days_to_expiry{host="example.com",port="443"} 157
```

---

## OCSF Format

The `ocsf` exporter maps QEM fields to the OCSF (Open Cybersecurity Schema
Framework) class 2004 — Vulnerability Finding.

### Field Mapping

| QEM Field | OCSF Field | Notes |
|-----------|------------|-------|
| `scan_id` | `finding_info.uid` | Scan event identifier |
| `finding.control_id` | `finding_info.uid` | Per-finding identifier |
| `finding.severity` | `severity_id` | 1=Info, 2=Low, 3=Medium, 4=High, 5=Critical |
| `finding.status` | `status` | Pass/Fail/Warn mapped to OCSF status |
| `target.host` | `resource.hostname` | Target hostname |
| `scanned_at` | `time` | Epoch milliseconds |
| `finding.title` | `title` | Finding title |
| `finding.detail` | `description` | Finding description |

### OCSF Severity Mapping

| QEM Severity | OCSF `severity_id` | OCSF Name |
|-------------|-------------------|-----------|
| Info | 1 | Informational |
| Low | 2 | Low |
| Medium | 3 | Medium |
| High | 4 | High |
| Critical | 5 | Critical |

**OCSF `class_uid` must be `2004`** (Vulnerability Finding).

---

## CEF Format

The `cef` exporter produces Common Event Format output suitable for syslog
and SIEM ingestion.

```
CEF:0|QuantX|QXScan|0.1|PCI-DSS-4.2.1|Strong TLS protocol required|1| \
  dvc=scanner01.example.com \
  dvchost=example.com \
  dpt=443 \
  cs1Label=standard cs1=pci-dss \
  cs2Label=controlId cs2=PCI-DSS-4.2.1 \
  cs3Label=status cs3=pass \
  cs4Label=severity cs4=critical \
  cs5Label=scanId cs5=a1b2c3d4-... \
  flexNumber1=100 flexNumber1Label=score \
  rt=Jun 27 2026 07:19:46 UTC
```

### CEF Field Mapping

| CEF Field | Source |
|-----------|--------|
| Device Vendor | `QuantX` (configurable) |
| Device Product | `QXScan` |
| Device Version | Binary version |
| Signature ID | `finding.control_id` |
| Name | `finding.title` |
| Severity | 1–10 mapped from Finding severity |
| `dvchost` | Target hostname |
| `dpt` | Target port |

---

## Enterprise Export Features

The following export features are **not in OSS** — they belong to
QX Enterprise:

| Feature | Description |
|---------|-------------|
| Elastic connector | Bidirectional continuous push + pull |
| Splunk HEC connector | Bidirectional |
| Datadog connector | Bidirectional |
| OpenTelemetry (OTLP) | Bidirectional |
| Continuous push | Retry, batching, compression, backpressure |
| Alert threshold engine | IF score < 90 THEN alert |
| PagerDuty / Slack / Teams / email | Notification channels |
