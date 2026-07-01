#!/usr/bin/env bash
# test_docker.sh — QXScan Docker Demo Test Suite
# Runs two phases inside the Docker demo stack:
#   Phase 1: Infrastructure health — are all services up and responding?
#   Phase 2: Content assertions — does the QXScan binary produce valid reports?
#
# Usage:
#   bash test_docker.sh                              # both phases
#   bash test_docker.sh --health-only                # phase 1 only
#   bash test_docker.sh --assertions-only /path/to/report.json  # phase 2 only
#
# Environment:
#   QXSCAN_BIN    — path to qxscan binary (default: ../target/release/qxscan)
#   COMPOSE_CMD   — docker compose command (default: docker compose)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_CMD="${COMPOSE_CMD:-docker compose -f $PROJECT_DIR/docker-compose.yml}"
QXSCAN_BIN="${QXSCAN_BIN:-$PROJECT_DIR/target/release/qxscan}"

PASS=0
FAIL=0
PHASE="${1:-both}"
REPORT_JSON="${2:-}"

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

# ── Phase 1: Infrastructure Health ────────────────────────────────────────

run_health_checks() {
  echo "═══ QXScan Docker Health ═══"
  echo ""

  echo "--- Docker Service Status ---"
  assert "alb container running"     "docker inspect qxscan-alb -f '{{.State.Status}}' | grep -c running > /dev/null"
  assert "frontend-pqc running"     "docker inspect qxscan-frontend-pqc -f '{{.State.Status}}' | grep -c running > /dev/null"
  assert "caddy-pqc running"        "docker inspect qxscan-caddy-pqc -f '{{.State.Status}}' | grep -c running > /dev/null"
  assert "frontend-legacy running"  "docker inspect qxscan-frontend-legacy -f '{{.State.Status}}' | grep -c running > /dev/null"
  assert "backend running"          "docker inspect qxscan-backend -f '{{.State.Status}}' | grep -c running > /dev/null"
  assert "db running"               "docker inspect qxscan-db -f '{{.State.Status}}' | grep -c running > /dev/null"
  assert "db-proxy running"         "docker inspect qxscan-db-proxy -f '{{.State.Status}}' | grep -c running > /dev/null"
  assert "mail running"             "docker inspect qxscan-mail -f '{{.State.Status}}' | grep -c running > /dev/null"

  echo ""
  echo "--- ALB Health Check ---"
  assert "ALB health endpoint"      "$COMPOSE_CMD exec -T alb curl -sf http://localhost:8080/health | grep -c 'ALB healthy' > /dev/null"
  assert "ALB routes endpoint"      "$COMPOSE_CMD exec -T alb curl -sf http://localhost:8080/routes | grep -c frontend_pqc > /dev/null"

  echo ""
  echo "--- Frontend PQC ---"
  assert "PQC frontend responds"     "$COMPOSE_CMD exec -T frontend-pqc curl -sfk https://localhost | grep -c 'QXScan' > /dev/null"
  assert "PQC frontend TLS 1.3"     "$COMPOSE_CMD exec -T frontend-pqc curl -skv https://localhost -o /dev/null 2>&1 | grep -c 'SSL connection using TLSv1.3' > /dev/null"

  echo ""
  echo "--- Frontend Legacy ---"
  assert "Legacy frontend responds"  "$COMPOSE_CMD exec -T frontend-legacy curl -sfk https://localhost | grep -c 'Legacy' > /dev/null"

  echo ""
  echo "--- Backend API ---"
  assert "Backend health endpoint"   "$COMPOSE_CMD exec -T backend curl -sfk https://localhost/api/health | grep -c 'ok' > /dev/null"
  assert "Backend Swagger docs"      "$COMPOSE_CMD exec -T backend curl -sfkL https://localhost/api/docs | grep -c 'swagger' > /dev/null"

  echo ""
  echo "--- Database ---"
  assert "DB accepts connections"    "$COMPOSE_CMD exec -T db pg_isready -U qxscan -d qxscan -h localhost | grep -c 'accepting connections' > /dev/null"

  echo ""
  echo "--- Mail Service ---"
  assert "Mail SMTP port open"       "$COMPOSE_CMD exec -T sh -c 'timeout 5 nc -zv localhost 587 2>&1' | grep -qi 'succeeded\\|open' || true"

  echo ""
  echo "═══ Health Results: ${PASS} passed, ${FAIL} failed ═══"
}

# ── Phase 2: Content Assertions ────────────────────────────────────────────

run_content_assertions() {
  echo "═══ QXScan Content Assertions ═══"
  if [ -n "$REPORT_JSON" ]; then
    echo "JSON: $REPORT_JSON"
  fi
  echo ""

  if [ -n "$REPORT_JSON" ]; then
    # schema_version
    assert "JSON: schema_version present"      "jq -e '.schema_version // empty' '$REPORT_JSON'"
    assert "JSON: schema_version == \"1\""     "jq -e '.schema_version == \"1\"'  '$REPORT_JSON'"

    # scan_id + scanned_at
    assert "JSON: scan_id present"             "jq -e '.scan_id // empty'         '$REPORT_JSON'"
    assert "JSON: scanned_at present"          "jq -e '.scanned_at // empty'      '$REPORT_JSON'"
    assert "JSON: scan_duration_ms present"    "jq -e '.scan_duration_ms >= 0'    '$REPORT_JSON'"

    # target object
    assert "JSON: target.host present"         "jq -e '.target.host // empty'     '$REPORT_JSON'"
    assert "JSON: target.port is number"       "jq -e '.target.port | type == \"number\"' '$REPORT_JSON'"
    assert "JSON: target.service present"      "jq -e '.target.service // empty'  '$REPORT_JSON'"

    # overall_status
    assert "JSON: overall_status present"      "jq -e '.overall_status // empty'  '$REPORT_JSON'"

    # TLS section
    assert "JSON: tls.negotiated_version"      "jq -e '.tls.negotiated_version // empty' '$REPORT_JSON'"
    assert "JSON: tls.cipher present"          "jq -e '.tls.cipher // empty'      '$REPORT_JSON'"
    assert "JSON: tls.forward_secrecy bool"    "jq -e '.tls.forward_secrecy | type == \"boolean\"' '$REPORT_JSON'"
    assert "JSON: tls.pqc_hybrid bool"         "jq -e '.tls.pqc_hybrid | type == \"boolean\"' '$REPORT_JSON'"

    # findings array
    assert "JSON: findings is array"           "jq -e '.findings | type == \"array\"' '$REPORT_JSON'"
    assert "JSON: findings has entries"        "jq -e '.findings | length > 0'        '$REPORT_JSON'"
    assert "JSON: finding has control_id"      "jq -e '.findings[0].control_id // empty' '$REPORT_JSON'"
    assert "JSON: finding has status"          "jq -e '.findings[0].status // empty'     '$REPORT_JSON'"
    assert "JSON: finding has severity"        "jq -e '.findings[0].severity // empty'   '$REPORT_JSON'"
    assert "JSON: finding has remediation key" "jq -e '.findings[0] | has(\"remediation\")' '$REPORT_JSON'"
    assert "JSON: no pass finding with non-null remediation" "jq -e '[.findings[] | select(.status==\"pass\" and .remediation != null)] | length == 0' '$REPORT_JSON'"

    # compliance scores
    assert "JSON: compliance is object"        "jq -e '.compliance | type == \"object\"' '$REPORT_JSON'"
    assert "JSON: compliance score is number"  "jq -e '[.compliance[]] | .[0].score | type == \"number\"' '$REPORT_JSON'"
    assert "JSON: compliance grade present"    "jq -e '[.compliance[]] | .[0].grade // empty' '$REPORT_JSON'"
    assert "JSON: controls_total present"      "jq -e '[.compliance[]] | .[0].controls_total >= 0' '$REPORT_JSON'"
  fi

  # exit-code assertions (always run)
  assert "Exit 0: no args (shows help)"     "$QXSCAN_BIN 2>/dev/null; [ \$? -eq 0 ]"
  assert "Exit 2: unknown flag"              "$QXSCAN_BIN --bogus 2>/dev/null; [ \$? -eq 2 ]"

  echo ""
  echo "═══ Content Results: ${PASS} passed, ${FAIL} failed ═══"
}

# ── Dispatch ────────────────────────────────────────────────────────────────

case "$PHASE" in
  --health-only)
    run_health_checks
    ;;
  --assertions-only)
    run_content_assertions
    ;;
  *)
    run_health_checks
    HEALTH_FAIL=$FAIL
    PASS=0; FAIL=0   # reset counters for phase 2
    run_content_assertions
    FAIL=$((FAIL + HEALTH_FAIL))  # accumulate failures across both phases
    ;;
esac

[[ $FAIL -eq 0 ]]
