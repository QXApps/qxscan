# Compliance Engine

> **Policy evaluation — pure functions, no side effects.**

---

## Overview

The compliance engine evaluates TLS evidence against regulatory standards
and produces findings. It is a pure function: given a `ScanEvent`, it
returns a list of `Finding` objects.

```
ScanEvent → compliance::scoring::evaluate(event, standard) → Vec<Finding>
                                                            → ComplianceScore
```

---

## Supported Standards

| Standard | CLI Slug | File | Controls |
|----------|----------|------|----------|
| PCI-DSS 4.0 | `pci-dss` | `pci_dss.rs` | 4 controls |
| HIPAA Security Rule | `hipaa` | `hipaa.rs` | 4 controls |
| SOC 2 Type II | `soc2` | `soc2.rs` | 4 controls |
| FISMA / NIST SP 800-52r2 | `fisma` | `fisma.rs` | 4 controls |
| CNSA 2.0 / PQC Readiness | `pqc` | `pqc.rs` | 3 controls |

### PCI-DSS 4.0 — Reference Standard

| Control ID | Check | Severity |
|------------|-------|----------|
| `PCI-DSS-4.2.1` | TLS version ≥ 1.2 (no SSL/early TLS) | Critical |
| `PCI-DSS-4.2.1.cipher` | Strong cipher (AES-256-GCM, AES-128-GCM, ChaCha20-Poly1305) | Critical |
| `PCI-DSS-4.2.1.fs` | Forward secrecy (ECDHE/DHE) | High |
| `PCI-DSS-4.2.1.1` | Certificate valid, not expired | Critical |

PCI-DSS is the reference standard — it exercises every TLS field and serves
as the template for all other standards.

### HIPAA Security Rule

| Control ID | Check | Severity |
|------------|-------|----------|
| `HIPAA-164.312.1` | TLS version ≥ 1.2 | Critical |
| `HIPAA-164.312.1.cipher` | Strong cipher | Critical |
| `HIPAA-164.312.1.fs` | Forward secrecy | High |
| `HIPAA-164.312.1.cert` | Certificate valid | Critical |

### SOC 2 Type II

| Control ID | Check | Severity |
|------------|-------|----------|
| `SOC2-CC6.1` | TLS version ≥ 1.2 | Critical |
| `SOC2-CC6.1.cipher` | Strong cipher | High |
| `SOC2-CC6.1.fs` | Forward secrecy | High |
| `SOC2-CC6.1.cert` | Certificate valid | Critical |

### FISMA / NIST SP 800-52r2

| Control ID | Check | Severity |
|------------|-------|----------|
| `FISMA-SP800-52r2.1` | TLS version ≥ 1.2 | Critical |
| `FISMA-SP800-52r2.1.cipher` | Strong cipher | Critical |
| `FISMA-SP800-52r2.1.fs` | Forward secrecy | High |
| `FISMA-SP800-52r2.1.cert` | Certificate valid | Critical |

### PQC — Post-Quantum Cryptography Readiness

| Control ID | Check | Severity |
|------------|-------|----------|
| `PQC-1.1` | PQC hybrid key exchange detected | High |
| `PQC-1.2` | TLS 1.3 negotiated | High |
| `PQC-1.3` | TLS 1.3 + PQC hybrid | High |

**PQC critical rule:** `pqc_hybrid == false` must produce `warn`, not `pass`.

---

## Control ID Format

```
{STANDARD}-{REFERENCE}.{CONTROL}[.{SUBCONTROL}]
```

Examples:
- `PCI-DSS-4.2.1` — TLS version check
- `PCI-DSS-4.2.1.cipher` — Cipher strength sub-check
- `HIPAA-164.312.1` — HIPAA access control
- `FISMA-SP800-52r2.1` — FISMA (NIST SP 800-52 Rev 2)
- `PQC-1.1` — PQC hybrid detection

---

## Finding Status

| Status | Meaning | Score Impact |
|--------|---------|--------------|
| `pass` | Control satisfied | Counts as passed |
| `fail` | Control violated | Counts as failed, may trigger exit code 3 |
| `warn` | Control partially satisfied | Counts as total (not passed) |
| `not_applicable` | Control not relevant | Excluded from scoring |

### Pass finding rule

A `pass` finding must NEVER have a non-null `remediation`. If remediation
guidance is needed, the status must be `warn` or `fail`.

---

## Severity Levels

| Severity | Scan Impact | Alerting |
|----------|-------------|----------|
| Critical | Fails scan (exit code 3) | Always alerted |
| High | Doesn't fail scan | Always alerted |
| Medium | Informational | Best-practice gap |
| Low | Informational | Minor gap |
| Info | Excluded from score | Observation only |

---

## Scoring Algorithm

### Formula (locked for v1)

```rust
score           = (controls_passed * 100) / controls_total   // u8, 0–100
controls_warned — counted in total but not in passed (no partial credit)
controls_total  — excludes NotApplicable findings
```

### Grade Table

| Grade | Score Range |
|-------|-------------|
| A+ | 100 |
| A | 90–99 |
| B | 80–89 |
| C | 70–79 |
| D | 60–69 |
| F | 0–59 |

### Implementation

```rust
// src/qem/compliance.rs
pub fn grade_from_score(score: u8) -> String {
    match score {
        100     => "A+".into(),
        90..=99 => "A".into(),
        80..=89 => "B".into(),
        70..=79 => "C".into(),
        60..=69 => "D".into(),
        _       => "F".into(),
    }
}
```

---

## Adding a New Control

1. Open `src/compliance/standards/<standard>.rs`
2. Push a `Finding` inside `evaluate()` following the existing pattern
3. Control ID: `{STANDARD}-{SECTION}.{CONTROL}`
4. `remediation: None` only when `status == FindingStatus::Pass`
5. Add a unit test: one pass case, one fail case

### Adding a New Standard

1. Create `src/compliance/standards/<name>.rs`
2. Export `pub fn evaluate(event: &ScanEvent) -> anyhow::Result<Vec<Finding>>`
3. Register in `standards/mod.rs` and `scoring.rs` match arm
4. Add CLI `--standards` enum variant in `cli/scan.rs`
5. Add pass + fail unit tests (see AGENTS.md for requirements)

### Unit Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{metadata::ScanEvent, observation::TlsInfo};

    fn make_event(tls_version: &str, pqc_hybrid: bool) -> ScanEvent {
        let mut e = ScanEvent::new(/* target */);
        e.tls = Some(TlsInfo {
            negotiated_version: tls_version.into(),
            cipher: "TLS_AES_256_GCM_SHA384".into(),
            forward_secrecy: true,
            pqc_hybrid,
            cert: None,
        });
        e
    }

    #[test]
    fn passes_on_tls13()  { /* assert pass */ }
    #[test]
    fn fails_on_tls10()   { /* assert fail */ }
}
```
