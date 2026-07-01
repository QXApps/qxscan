# QXScan Demo Suite

Full-stack, TLS-secured demo environment for testing QXScan's TLS security and compliance scanning capabilities. Runs entirely via Docker Compose.

## Architecture

```
                         ┌──────────────────┐
                         │     ALB (8443)   │
                         │  Nginx LB Proxy  │
                         └──┬──────────┬───┘
                            │          │
                 ┌──────────▼──┐  ┌────▼──────────┐
                 │ frontend-pqc│  │frontend-legacy │
                 │ TLS 1.3+PQC │  │  TLS 1.2 only │
                 └──────┬──────┘  └────┬──────────┘
                        │              │
                 ┌──────▼──────────────▼────┐
                 │      Backend API        │
                 │  Node.js + Swagger      │
                 │   TLS 1.3 + PQC         │
                 └──────┬────────────┬─────┘
                        │            │
                    ┌───▼──┐    ┌────▼──────┐
                    │  DB  │    │   Mail    │
                    │ PG16 │    │  SMTP TLS │
                    └──────┘    └───────────┘

         ┌─────────────────────────────────┐
         │         caddy-pqc               │
         │  Dedicated PQC fixture (Caddy)  │
         │  TLS 1.3 + X25519MLKEM768       │
         └─────────────────────────────────┘
```

### Services

| Service | TLS | PQC | Purpose |
|---|---|---|---|
| **alb** (port 8443) | TLS 1.2/1.3 | ✅ PQC-ready | ALB reverse proxy — routes to both frontends |
| **frontend-pqc** | TLS 1.3 only | ✅ X25519MLKEM768 | PQC-ready web frontend with dashboard |
| **caddy-pqc** | TLS 1.3 | ✅ X25519MLKEM768 | Canonical PQC fixture (Caddy v2) |
| **frontend-legacy** | TLS 1.2 only | ❌ No PQC | Legacy target for compliance failure testing |
| **backend** | TLS 1.3 | ✅ PQC-ready | Express API with Swagger docs + stub endpoints |
| **db** | STARTTLS | ✅ PQC-ready | PostgreSQL 16 with TLS certs |
| **db-proxy** (port 5433) | Direct TLS | ✅ PQC-ready | socat TLS proxy → PostgreSQL STARTTLS |
| **mail** (port 587) | STARTTLS | ✅ PQC-ready | SMTP stub — accepts and logs emails |
| **scanner** | — | — | Ad-hoc QXScan CLI |
| **qxscaner** | — | — | QXScan daemon (server mode) |

## Quick Start

```bash
# Build all images (from project root)
docker compose build

# Start full stack
docker compose up -d

# Check services are healthy
docker compose ps

# Run individual scan
docker compose run --rm scanner scan frontend-pqc --port 443 --no-verify --timeout 15 --standards pqc,pci-dss

# Run full demo scan suite
docker compose up demo-scan
```

## Scanning Strategies

### 1. Scan via ALB (mimicking real-world ingress)

```bash
docker compose run --rm scanner scan alb --port 443 --no-verify --timeout 15
```

### 2. Scan the canonical PQC fixture

```bash
docker compose run --rm scanner scan caddy-pqc --port 443 --no-verify --timeout 15 --standards pqc
```

### 3. Compare PQC vs legacy

```bash
# PQC-ready — should pass
docker compose run --rm scanner scan frontend-pqc --port 443 --standards pqc

# Legacy — should fail PQC checks
docker compose run --rm scanner scan frontend-legacy --port 443 --standards pqc
```

## Makefile Targets

```bash
cd demo && make up          # Start full stack
cd demo && make scan-all    # Scan all services
cd demo && make pqc-check   # PQC readiness scan
cd demo && make legacy-check # Verify legacy detection
cd demo && make status      # Container status
cd demo && make down        # Stop everything
```

## Test Suite

The demo folder includes a layered test harness for Docker health, report
schema, and scanner classification correctness.

### `test_docker.sh` — Docker health + content assertions

Single script covering both Docker infrastructure and report validation.

```bash
cd demo && bash test_docker.sh                              # both phases
cd demo && bash test_docker.sh --health-only                # infrastructure only
cd demo && bash test_docker.sh --assertions-only /tmp/report.json  # content only
```

**Phase 1 — Infrastructure Health:** All 8 containers running, ALB health
endpoints, frontend HTTPS responses, backend API + Swagger, DB connectivity,
SMTP port open. (17 assertions, requires Docker.)

**Phase 2 — Content Assertions:** JSON schema validation via `jq` (schema
version, scan metadata, TLS fields, findings array, compliance scores),
plus exit code checks for 0 and 2. (Optional JSON report argument.)

### `classification_test.sh` — External target classification

Scans external hosts and reports how each was classified.

```bash
cd demo && bash classification_test.sh standard   # ~60 popular HTTPS sites
cd demo && bash classification_test.sh services   # non-443 TLS services
cd demo && bash classification_test.sh edge       # deterministic edge cases
cd demo && bash classification_test.sh all        # all three sequentially
```

### `test_all.sh` — Master test runner

Orchestrates all suites with optional filtering.

```bash
cd demo && bash test_all.sh                  # all suites
cd demo && bash test_all.sh --skip-docker    # skip Docker-dependent checks
cd demo && bash test_all.sh --suite edge     # only the edge-case suite
```

Environment variables:

| Variable | Default | Purpose |
|----------|---------|---------|
| `QXSCAN_BIN` | `../target/release/qxscan` | Path to the qxscan binary |
| `QXSCAN_TIMEOUT` | `5` | Seconds per classification target |
| `SKIP_DOCKER` | `0` | Set to `1` to skip Docker suites |

You can also invoke suites via the Makefile:

```bash
cd demo && make test-all             # all suites (via test_all.sh)
cd demo && make test-classification  # standard targets
cd demo && make test-services        # service protocol targets
cd demo && make test-edge            # edge-case targets
cd demo && make clean-results        # remove stale scan artifacts
```

### Target files

All scan target lists live under `demo/targets/`:

| File | Purpose |
|------|---------|
| `targets-docker.txt` | Docker demo services (frontend-pqc, caddy-pqc, backend, …) |
| `targets-standard.txt` | ~60 popular HTTPS sites for broad classification testing |
| `targets-services.txt` | Non-443 TLS services (DNS-over-TLS, SMTPS, IMAPS, APIs) |
| `targets-edge.txt` | Deterministic targets with documented expected outcomes for CI |

## CI Integration

The GitHub Actions workflow (`.github/workflows/ci.yml`) automatically runs
the content assertion suite on every push and PR:

1. Builds the release binary
2. Scans `example.com` and saves a JSON report
3. Runs `test_docker.sh --assertions-only` against the report — validates JSON
   schema, exit codes, and finding structure with `jq`
4. If the network scan fails, a fallback step still checks exit codes 0 and 2
   to ensure the binary is functional

## Certificate Details

- **PQC-ready services**: ECDSA (prime256v1) certificates — compatible with X25519MLKEM768
- **Legacy service**: RSA 2048-bit certificates — TLS 1.2 only
- All certificates are self-signed by the demo CA (valid 10 years)
- Generated via `demo/certs/generate.sh`

## Testing PQC Detection

QXScan detects PQC readiness through:
1. Negotiated Cipher Suite IDs (TLS 1.3 ciphers)
2. `key_share` extension analysis (detects X25519MLKEM768, X25519Kyber768)
3. TLS version negotiation

Compare results between:
- `caddy-pqc` / `frontend-pqc` → `pqc_hybrid: true`
- `frontend-legacy` → `pqc_hybrid: false`, PQC compliance failure
