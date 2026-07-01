# Contributing to QXScan

> **Thank you for contributing!** This guide covers the coding conventions,
> contribution process, and checklist for all changes.

---

## Table of Contents

1. [Code of Conduct](#1-code-of-conduct)
2. [Getting Started](#2-getting-started)
3. [Coding Conventions](#3-coding-conventions)
4. [Naming Conventions](#4-naming-conventions)
5. [Error Messages](#5-error-messages)
6. [Testing](#6-testing)
7. [Contribution Checklist](#7-contribution-checklist)
8. [What NOT To Do](#8-what-not-to-do)

---

## 1. Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](../CODE_OF_CONDUCT.md).
Please read it before participating. Be respectful, constructive, and professional.
This is a security tool — quality matters more than velocity.

---

## 2. Getting Started

```bash
# Clone the repo
git clone https://github.com/QXApps/qxscan.git
cd qxscan

# Build
cargo build --release

# Run tests
cargo test

# Verify lints
cargo clippy -- -D warnings
cargo fmt --check
```

---

## 3. Coding Conventions

### Hard Rules

- **No `unwrap()` or `expect()`** in library code (scanner, tls, compliance,
  export, event modules). Use `?` and `anyhow::bail!()`.
  Permitted only in tests and `main.rs`.
- **No `println!`** in library modules. Use `log::debug!` / `info!` / `warn!`.
- **No `eprintln!`** outside `cli/` modules.
- Only `report/` and `cli/` modules write to stdout/stderr.
- Return `anyhow::Result<T>` from all fallible functions.
- Prefer `thiserror` for domain errors when callers need to match.

### Module Structure

Each module has a clear responsibility boundary:

| Module | Side Effects? | Description |
|--------|--------------|-------------|
| `scanner/`, `tls/` | Pure functions | No I/O, no state |
| `compliance/` | Pure functions | No network, no I/O |
| `qem/` | Data types | Serialize/Deserialize only |
| `export/` | Read-only | Consumes ScanEvent, never mutates |
| `cli/` | Dispatch only | CLI parsing + delegation |
| `server/` | I/O | Database, PID file |
| `schedule/` | I/O | Cron evaluation, scan execution |

### Import Style

- Group imports: `std` → external crates → `crate::`
- Alphabetical within groups
- No wildcard imports (`use crate::*`)

### Error Handling

```rust
// ❌ Wrong — unclear, not actionable
error: failed to open file

// ✅ Correct — specific, actionable
error: cannot open targets file '/tmp/hosts.txt': No such file or directory
hint:  create the file or pass a different --targets-file path
```

---

## 4. Naming Conventions

| Thing | Convention | Example |
|-------|-----------|---------|
| Standard slug (CLI/JSON) | kebab-case | `pci-dss`, `soc2`, `hipaa` |
| Standard slug (Rust field/key) | snake_case | `pci_dss`, `hipaa` |
| Control ID | UPPER-KEBAB dot notation | `PCI-DSS-4.2.1` |
| Metric names | `qxscan_` prefix + snake_case | `qxscan_compliance_score` |
| Config keys | snake_case | `timeout_s`, `bind` |
| Types | PascalCase | `ScanEvent`, `TlsInfo` |
| Functions | snake_case | `grade_from_score()`, `init_pool()` |
| Variables | snake_case | `scan_id`, `target_host` |

---

## 5. Error Messages

Error messages must be **actionable**:

```rust
// ❌ Wrong — doesn't help the user
bail!("failed to open file");

// ✅ Correct — tells the user what and how to fix
bail!(
    "cannot open targets file '{}': No such file or directory\n\
     hint: create the file or pass a different --targets-file path",
    path.display()
);
```

---

## 6. Testing

### Unit Tests

Every compliance standard must have **≥1 pass test and ≥1 fail test**:

```rust
#[test]
fn standard_x_passes() {
    let findings = evaluate(&make_pass_event()).unwrap();
    assert!(findings.iter().any(|f| f.status == FindingStatus::Pass));
}

#[test]
fn standard_x_fails() {
    let findings = evaluate(&make_fail_event()).unwrap();
    assert!(findings.iter().any(|f| f.status == FindingStatus::Fail));
}
```

### Database Tests

For tests requiring a database, use a single-connection in-memory SQLite
pool with raw DDL table creation:

```rust
async fn test_pool() -> SqlitePool {
    let opts = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1) // in-memory is per-connection
        .connect_with(opts).await.unwrap();
    // Create tables...
    pool
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test compliance

# Run with output
cargo test -- --nocapture

# Lints
cargo clippy -- -D warnings
cargo fmt --check

# Security audit
cargo audit
```

### Demo Suite

The project includes a layered test harness under `demo/`:

```bash
# Docker health + content assertions
cd demo && bash test_docker.sh

# Classification tests against external targets
cd demo && bash classification_test.sh edge

# Master test runner (all suites)
cd demo && bash test_all.sh
```

See [`docs/testing.md`](testing.md) for full details on the test suite.

---

## 7. Contribution Checklist

Before submitting a pull request, verify all of the following:

```
[ ] cargo build --release succeeds, zero warnings
[ ] cargo test — all green
[ ] cargo clippy -- -D warnings — clean
[ ] cargo fmt --check — clean
[ ] cargo audit — no new vulnerabilities
[ ] demo/test_docker.sh — all assertions PASS (if applicable)
[ ] demo/classification_test.sh — classification results as expected
[ ] New controls have pass + fail unit tests
[ ] No pass finding has non-null remediation
[ ] PQC controls: pqc_hybrid=false → warn, not pass
[ ] ScanEvent schema_version unchanged (or bumped with migration)
[ ] No unwrap()/expect() added in library code
[ ] No credentials or secrets in any committed file
[ ] No enterprise features added to OSS modules
[ ] Known issues not made worse
```

---

## 8. What NOT To Do

```
❌ No unwrap()/expect() in library code
❌ No network calls inside compliance/standards/
❌ No credentials or API keys in source code or config values
   → use env var syntax: "${MY_ENV_VAR}"
❌ No ScanEvent field name/type changes without schema_version bump
❌ No enterprise features:
   - No Elastic/Splunk/Datadog/OTLP connectors
   - No continuous push, streaming, retry, batching
   - No alerting rules (IF score < X THEN alert)
   - No dashboards or web UI
   - No RBAC, SSO, OIDC, SAML, LDAP
   - No multi-tenant or multi-node coordination
❌ No clap imports outside src/cli/
❌ No println!/eprintln! in scanner, tls, compliance, export
❌ No new standard without pass + fail unit tests
❌ No hardcoded cron expressions outside src/schedule/cron.rs
❌ No hardcoded port numbers outside src/scanner/service.rs
❌ No inline SQLite schema outside src/server/state.rs
❌ Do not make QEM proprietary or move it behind a license gate
```

---

*QXScan OSS — Apache 2.0 License*
