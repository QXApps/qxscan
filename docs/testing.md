# Testing

> **How to build, test, and validate QXScan.**

---

## Quick Start

```bash
# Full validation suite
cargo build --release
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

---

## Building

```bash
cargo build                     # Debug build
cargo build --release           # Production build
strip target/release/qxscan     # Reduce binary size (target: < 10 MB)
```

### Docker

```bash
# Build scanner image
docker compose build scanner

# Start demo infrastructure
docker compose up -d

# Run a scan inside container
docker compose run --rm scanner scan frontend-pqc --port 443 --no-verify
```

---

## Unit Tests

```bash
# Run all tests
cargo test

# Run specific module
cargo test compliance
cargo test server::state
cargo test schedule::cron

# Run with verbose output
cargo test -- --nocapture
```

### Test Coverage

| Module | Tests | What's Tested |
|--------|-------|---------------|
| `compliance/standards/pci_dss.rs` | 3 | Pass + fail + warn scenarios |
| `compliance/standards/hipaa.rs` | 3 | Pass + fail + warn scenarios |
| `compliance/standards/soc2.rs` | 3 | Pass + fail + warn scenarios |
| `compliance/standards/fisma.rs` | 3 | Pass + fail + warn scenarios |
| `compliance/standards/pqc.rs` | 3 | Pass + fail + PQC rules |
| `schedule/cron.rs` | 2 | Daily + hourly cron computation |
| `server/state.rs` | 9 | Report CRUD (insert, list, filter, get, delete) |
| `tls/handshake.rs` | 10 | TLS probe: connection, handshake, classification |
| `scanner/mod.rs` | 8 | Target resolution, probe orchestration |
| `qem/` (event tests) | 12 | QEM type construction, serialization, status handling |

**Total: 73 tests**

### Writing Tests

Every compliance standard must have **≥1 pass test and ≥1 fail test**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::qem::{metadata::ScanEvent, observation::TlsInfo};

    fn make_event(tls_version: &str) -> ScanEvent {
        let mut e = ScanEvent::new(/* target */);
        e.tls = Some(TlsInfo {
            negotiated_version: tls_version.into(),
            cipher: "TLS_AES_256_GCM_SHA384".into(),
            forward_secrecy: true,
            pqc_hybrid: false,
            cert: None,
        });
        e
    }

    #[test]
    fn passes_on_tls13() {
        let findings = evaluate(&make_event("TLSv1.3")).unwrap();
        assert!(findings.iter().any(|f| f.status == FindingStatus::Pass));
    }

    #[test]
    fn fails_on_sslv3() {
        let findings = evaluate(&make_event("SSLv3")).unwrap();
        assert!(findings.iter().any(|f| f.status == FindingStatus::Fail));
    }
}
```

### Database Tests

For tests requiring a database, use a single-connection in-memory SQLite
pool. This avoids filesystem dependencies and ensures test isolation:

```rust
async fn test_pool() -> SqlitePool {
    let opts = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1) // :memory: is per-connection
        .connect_with(opts).await.unwrap();
    // Create test tables...
    pool
}
```

**Key insight:** SQLite's `:memory:` databases are per-connection.
Using `max_connections(1)` ensures all queries within a test share the
same database.

---

## Lints & Formatting

```bash
# Clippy lints (CI enforced — must pass)
cargo clippy -- -D warnings

# Formatting (CI enforced — must pass)
cargo fmt --check

# Auto-fix formatting
cargo fmt
```

---

## Security Audit

```bash
# Check dependency vulnerabilities
cargo audit
```

If `cargo-audit` is not installed:
```bash
cargo install cargo-audit
```

---

## Demo Suite

The project includes a layered test harness under `demo/` that validates
Docker infrastructure, report schema, and scanner classification correctness.

### Setup

```bash
# Build the scanner image with latest code
docker compose build scanner

# Start demo services (PQC fixtures, legacy TLS, etc.)
docker compose up -d
```

### Test Scripts

```bash
# Infrastructure health + content assertions
cd demo && bash test_docker.sh

# Infrastructure health only
cd demo && bash test_docker.sh --health-only

# Content assertions only (requires a JSON report)
cd demo && bash test_docker.sh --assertions-only /tmp/report.json

# Classification tests against external targets
cd demo && bash classification_test.sh standard   # ~60 popular HTTPS sites
cd demo && bash classification_test.sh services   # non-443 TLS services
cd demo && bash classification_test.sh edge       # deterministic edge cases

# Master test runner (all suites)
cd demo && bash test_all.sh

# Skip Docker-dependent checks
cd demo && bash test_all.sh --skip-docker

# Via Makefile
cd demo && make test-all
cd demo && make test-classification
cd demo && make test-services
cd demo && make test-edge
```

### What's tested

| Suite | Script | What it validates |
|-------|--------|-------------------|
| Docker health | `test_docker.sh --health-only` | 8 containers running, ALB health, frontend HTTPS, backend API, DB, SMTP |
| Content assertions | `test_docker.sh --assertions-only` | JSON schema (jq), exit codes, finding structure, compliance scores |
| Classification | `classification_test.sh` | External target classification (pass/fail/warn/timeout/no-tls) |
| Master runner | `test_all.sh` | Orchestrates all suites with optional filtering |

### Full Demo Scan

```bash
docker compose up demo-scan
```

This runs scans against all 7 demo targets:
- `frontend-pqc` — TLS 1.3 + PQC (expected: Pass)
- `caddy-pqc` — Dedicated PQC fixture (expected: Pass)
- `frontend-legacy` — TLS 1.2 only (expected: Fail — cipher failures)
- `alb` — Load balancer (expected: Pass)
- `backend` — API endpoint (expected: Pass)
- `db-proxy` — Direct TLS on port 5433 (expected: Pass)
- `mail` — SMTP STARTTLS (expected: Pass)

---

## Continuous Integration

CI enforces the following checks:

```yaml
# .github/workflows/ci.yml (if configured)
- cargo build --release
- cargo test
- cargo clippy -- -D warnings
- cargo fmt --check
- cargo audit
```

---

## Troubleshooting

### Build fails with "No such file or directory" for migrations

Ensure the `migrations/` directory exists at the project root with the
required `.sql` files. The `sqlx::migrate!` macro embeds these at compile
time.

### Database "no such table" in tests

In-memory SQLite databases are per-connection. Use `max_connections(1)`
when creating `SqlitePool` for tests, or use a temporary file.

### Docker build fails

Ensure `migrations/` is included in the Docker build context:
```dockerfile
COPY migrations/ ./migrations/
```
