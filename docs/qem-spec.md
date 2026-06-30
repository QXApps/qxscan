# QEM — QX Event Model Specification

> **Version 1 — Stable**
>
> QEM (QX Event Model) is the **open, versioned observation format** at the
> heart of QXScan. It is the contract between scanner engines, compliance
> engines, export formats, state stores, and third-party tools.
>
> **QEM is permanently open source.** It may never become proprietary.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Schema](#2-schema)
3. [ScanEvent Envelope](#3-scanevent-envelope)
4. [Evidence Layer](#4-evidence-layer)
5. [Observation Layer](#5-observation-layer)
6. [Compliance Layer](#6-compliance-layer)
7. [Scoring Algorithm](#7-scoring-algorithm)
8. [JSON Example](#8-json-example)
9. [Schema Change Rules](#9-schema-change-rules)
10. [PQC Scoring Rules](#10-pqc-scoring-rules)

---

## 1. Overview

QEM is a JSON-based event format produced by TLS security scanners. It
captures the full result of a single scan in a structured, machine-readable
document:

```
ScanEvent (v1)
├── metadata       — scan_id, timestamp, target, overall status
├── evidence       — TLS version, cipher suite, certificate info
├── findings       — compliance control evaluations (pass/fail/warn)
└── compliance     — rolled-up scores per standard
```

Any tool can produce or consume QEM. The format is versioned and
backward-compatible within the same major version.

---

## 2. Schema

### Namespaces

| Namespace | File (src/qem/) | Description |
|-----------|-------------------|-------------|
| `metadata` | `metadata.rs` | ScanEvent envelope, target, status |
| `evidence` | `observation.rs` | Raw wire data (TlsInfo, CertInfo) |
| `findings` | `finding.rs` | Interpreted conclusions per control |
| `compliance` | `compliance.rs` | Rolled-up scores per standard |

### Version string

The current schema version is `"1"`. Any breaking change requires bumping
this to `"2"` and providing a migration path.

---

## 3. ScanEvent Envelope

### `ScanEvent`

```json
{
  "schema_version":   "1",
  "scan_id":          "<uuid-v4>",
  "scanned_at":       "<rfc3339-utc>",
  "scan_duration_ms": <u64>,
  "target":           { <TargetInfo> },
  "overall_status":   "<pass|fail|warn|error|timeout|...>",
  "tls":              <TlsInfo | null>,
  "findings":         [ <Finding>, ... ],
  "compliance":       { "<standard>": <ComplianceScore>, ... }
}
```

### `TargetInfo`

| Field | Type | Description |
|-------|------|-------------|
| `host` | string | Target hostname or IP |
| `ip` | string | Resolved IP address |
| `port` | u16 | TCP port |
| `service` | string | Service type (https, smtp, etc.) |

### `ScanStatus` values

| Value | Meaning |
|-------|---------|
| `pass` | All controls passed |
| `fail` | At least one FAIL finding |
| `warn` | At least one WARN finding, no FAIL findings |
| `error` | Probe or evaluation error |
| `timeout` | Connection timed out |
| `notls` | Connected but no TLS detected |
| `unsupported_protocol` | Target doesn't support TLS 1.2+ |
| `connection_failed` | Could not connect |

---

## 4. Evidence Layer

### `TlsInfo`

| Field | Type | Description |
|-------|------|-------------|
| `negotiated_version` | string | e.g. `TLSv1.3`, `TLSv1.2` |
| `cipher` | string | Full cipher suite name (e.g. `TLS_AES_256_GCM_SHA384`) |
| `forward_secrecy` | bool | ECDHE or DHE key exchange |
| `pqc_hybrid` | bool | Post-quantum hybrid key exchange detected |
| `cert` | CertInfo or null | Leaf certificate details |

### `CertInfo`

| Field | Type | Description |
|-------|------|-------------|
| `subject` | string | Certificate subject DN |
| `issuer` | string | Certificate issuer DN |
| `not_before` | string | Validity start (RFC 3339 UTC) |
| `not_after` | string | Validity end (RFC 3339 UTC) |
| `days_to_expiry` | u64 | Days until certificate expiry |
| `san` | string[] | Subject Alternative Names |
| `serial` | string | Certificate serial number |

---

## 5. Observation Layer

### `Finding`

| Field | Type | Description |
|-------|------|-------------|
| `control_id` | string | Unique control identifier (e.g. `PCI-DSS-4.2.1`) |
| `standard` | string | Standard slug (e.g. `pci-dss`, `hipaa`) |
| `status` | string | `pass`, `fail`, `warn`, or `not_applicable` |
| `severity` | string | `info`, `low`, `medium`, `high`, or `critical` |
| `title` | string | Short human-readable summary |
| `detail` | string | Detailed explanation with evidence |
| `remediation` | string or null | Remediation guidance (null when status=pass) |

### `Severity` meanings

| Severity | Impact |
|----------|--------|
| `critical` | Fails the scan (exit code 3). Always alerted. |
| `high` | Always alerted. Counts toward score. |
| `medium` | Doesn't fail the scan. Counts toward score. |
| `low` | Best-practice gap. Counts toward score. |
| `info` | Observation only. NOT counted toward score. |

### Rule: Pass findings MUST NOT have remediation

A finding with `status: "pass"` must have `remediation: null`. If
remediation guidance exists, the status must be `"warn"` or `"fail"`.

---

## 6. Compliance Layer

### `ComplianceScore`

| Field | Type | Description |
|-------|------|-------------|
| `score` | u8 (0–100) | Numeric score |
| `grade` | string | Letter grade (A+ through F) |
| `controls_passed` | u32 | Number of passing controls |
| `controls_failed` | u32 | Number of failing controls |
| `controls_warned` | u32 | Number of warned controls |
| `controls_total` | u32 | Total applicable controls |

---

## 7. Scoring Algorithm

### Formula (locked for v1)

```
score           = (controls_passed * 100) / controls_total   [u8, 0–100]
controls_warned — counted in total but not in passed (no partial credit)
controls_total  — excludes NotApplicable findings
```

### Grade table

| Grade | Score |
|-------|-------|
| A+ | 100 |
| A | 90–99 |
| B | 80–89 |
| C | 70–79 |
| D | 60–69 |
| F | < 60 |

---

## 8. JSON Example

```json
{
  "schema_version": "1",
  "scan_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "scanned_at": "2026-06-27T07:19:46Z",
  "scan_duration_ms": 57,
  "target": {
    "host": "example.com",
    "ip": "93.184.216.34",
    "port": 443,
    "service": "https"
  },
  "overall_status": "pass",
  "tls": {
    "negotiated_version": "TLSv1.3",
    "cipher": "TLS_AES_256_GCM_SHA384",
    "forward_secrecy": true,
    "pqc_hybrid": false,
    "cert": {
      "subject": "CN=example.com",
      "issuer": "CN=CA, O=Example Inc.",
      "not_before": "2025-12-01T00:00:00Z",
      "not_after": "2026-12-01T00:00:00Z",
      "days_to_expiry": 157,
      "san": ["example.com", "www.example.com"],
      "serial": "04:AB:CD:EF:01:23:45:67"
    }
  },
  "findings": [
    {
      "control_id": "PCI-DSS-4.2.1",
      "standard": "pci-dss",
      "status": "pass",
      "severity": "critical",
      "title": "Strong TLS protocol required",
      "detail": "Negotiated: TLSv1.3 (TLS_AES_256_GCM_SHA384)",
      "remediation": null
    },
    {
      "control_id": "PCI-DSS-4.2.1.fs",
      "standard": "pci-dss",
      "status": "pass",
      "severity": "high",
      "title": "Forward secrecy required",
      "detail": "Key exchange uses ECDHE",
      "remediation": null
    }
  ],
  "compliance": {
    "pci_dss": {
      "score": 100,
      "grade": "A+",
      "controls_passed": 4,
      "controls_failed": 0,
      "controls_warned": 0,
      "controls_total": 4
    }
  }
}
```

---

## 9. Schema Change Rules

| Change | Allowed? | Required |
|--------|----------|----------|
| Add optional field (`Option<T>`) | ✅ | `#[serde(skip_serializing_if = "Option::is_none")]` |
| Add field with default | ✅ | `#[serde(default)]` |
| Rename field | ❌ | Use `#[serde(rename)]` if unavoidable |
| Remove field | ❌ | Mark `#[serde(skip)]` first, deprecate, remove later |
| Change field type | ❌ | Add new field, keep old for compatibility |
| Any breaking change | ❌ | Bump `schema_version` to `"2"`, write migration guide |

### Serialization rules

- **Correct (structured, machine-readable):**
  ```json
  "pci_dss": { "score": 100, "grade": "A+", "controls_passed": 4, ... }
  ```

- **Wrong (prose string, breaks consumers):**
  ```json
  "pci_dss": "A+ (100)"
  ```

---

## 10. PQC Scoring Rules

These rules are critical for correct post-quantum cryptography assessment:

| `pqc_hybrid` | TLS Version | PQC-1.1 | PQC-1.3 |
|--------------|-------------|---------|---------|
| `true` | 1.3 | pass | pass |
| `false` | 1.3 | **warn** | **warn** |
| `false` | 1.2 | **warn** | **fail** |

**Key rule:** `pqc_hybrid == false` must produce `warn`, not `pass`.
A server without PQC hybrid key exchange is not quantum-ready.

---

## Control ID Format

```
{STANDARD}-{REFERENCE}.{CONTROL}[.{SUBCONTROL}]
```

Examples:
- `PCI-DSS-4.2.1`
- `PCI-DSS-4.2.1.cipher`
- `HIPAA-164.312.1`
- `FISMA-SP800-52r2.1`
- `PQC-1.1`

---

*QEM v1 — Open specification. Apache 2.0 License.*
*Maintained at https://github.com/QXApps/qxscan*
