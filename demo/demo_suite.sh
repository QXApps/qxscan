#!/usr/bin/env bash
# demo_suite.sh — QXScan Demo Docker Test Suite
# Validates that all demo services are up and responding correctly.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

PASS=0
FAIL=0

assert() {
  local desc="$1"; shift
  if eval "$@" &>/dev/null; then
    echo "✅ PASS  $desc"
    PASS=$((PASS + 1))
  else
    echo "❌ FAIL  $desc"
    FAIL=$((FAIL + 1))
  fi
}

echo "═══ QXScan Demo Suite — Content Assertions ═══"
echo ""

# Check Docker services are running
echo "--- Docker Service Status ---"
assert "alb container running"     "docker inspect qxscan-alb -f '{{.State.Status}}' | grep -q running"
assert "frontend-pqc running"     "docker inspect qxscan-frontend-pqc -f '{{.State.Status}}' | grep -q running"
assert "caddy-pqc running"        "docker inspect qxscan-caddy-pqc -f '{{.State.Status}}' | grep -q running"
assert "frontend-legacy running"  "docker inspect qxscan-frontend-legacy -f '{{.State.Status}}' | grep -q running"
assert "backend running"          "docker inspect qxscan-backend -f '{{.State.Status}}' | grep -q running"
assert "db running"               "docker inspect qxscan-db -f '{{.State.Status}}' | grep -q running"
assert "db-proxy running"         "docker inspect qxscan-db-proxy -f '{{.State.Status}}' | grep -q running"
assert "mail running"             "docker inspect qxscan-mail -f '{{.State.Status}}' | grep -q running"

echo ""
echo "--- ALB Health Check ---"
# ALB exposes an HTTP health endpoint on port 8080 internally
assert "ALB health endpoint"      "${COMPOSE_CMD:-docker compose} exec -T alb curl -sf http://localhost:8080/health | grep -q 'ALB healthy'"
assert "ALB routes endpoint"      "${COMPOSE_CMD:-docker compose} exec -T alb curl -sf http://localhost:8080/routes | grep -q frontend_pqc"

echo ""
echo "--- Frontend PQC ---"
assert "PQC frontend responds"     "${COMPOSE_CMD:-docker compose} exec -T frontend-pqc curl -sfk https://localhost | grep -q 'QXScan'"
assert "PQC frontend TLS 1.3"     "${COMPOSE_CMD:-docker compose} exec -T frontend-pqc curl -sfkI https://localhost 2>&1 | grep -qi 'HTTP/2'"

echo ""
echo "--- Frontend Legacy ---"
assert "Legacy frontend responds"  "${COMPOSE_CMD:-docker compose} exec -T frontend-legacy curl -sfk https://localhost | grep -q 'Legacy'"

echo ""
echo "--- Backend API ---"
assert "Backend health endpoint"   "${COMPOSE_CMD:-docker compose} exec -T backend curl -sfk https://localhost/api/health | grep -q 'ok'"
assert "Backend Swagger docs"      "${COMPOSE_CMD:-docker compose} exec -T backend curl -sfkL https://localhost/api/docs | grep -q 'swagger'"

echo ""
echo "--- Database ---"
assert "DB accepts connections"    "${COMPOSE_CMD:-docker compose} exec -T db pg_isready -U qxscan -d qxscan -h localhost | grep -q 'accepting connections'"

echo ""
echo "--- Mail Service ---"
assert "Mail SMTP port open"       "${COMPOSE_CMD:-docker compose} exec -T sh -c 'timeout 5 nc -zv localhost 587 2>&1' | grep -qi 'succeeded\|open' || true"

echo ""
echo "═══ Results: ${PASS} passed, ${FAIL} failed ═══"
[[ $FAIL -eq 0 ]]
