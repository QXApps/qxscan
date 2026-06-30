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
