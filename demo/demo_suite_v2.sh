#!/usr/bin/env bash
# demo_suite_v2.sh — QXScan OSS content assertion test suite.
# Validates JSON/HTML report structure using jq assertions.
set -uo pipefail

REPORT_JSON="${1:-}"
REPORT_HTML="${2:-}"
PASS=0; FAIL=0

assert() {
  local desc="$1"; shift
  if eval "$@" &>/dev/null; then
    echo "✅ PASS  $desc"; ((PASS++))
  else
    echo "❌ FAIL  $desc"; ((FAIL++))
  fi
}

echo "═══ QXScan Content Assertions ═══"
if [ -n "$REPORT_JSON" ]; then
  echo "JSON: $REPORT_JSON"
fi
if [ -n "$REPORT_HTML" ]; then
  echo "HTML: $REPORT_HTML"
fi
echo ""

# ─── JSON Schema Assertions (single ScanEvent object, not array) ───
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

# exit-code assertions
assert "Exit 0: no args (shows help)"     "./target/release/qxscan 2>/dev/null; [ \$? -eq 0 ]"
assert "Exit 2: unknown flag"              "./target/release/qxscan --bogus 2>/dev/null; [ \$? -eq 2 ]"

# HTML sanity (optional — skip if no html report provided)
if [ -n "$REPORT_HTML" ]; then
  assert "HTML: DOCTYPE present"             "grep -q 'DOCTYPE html' '$REPORT_HTML'"
  assert "HTML: scan_id in HTML"             "grep -q 'scan_id\\|Scan ID' '$REPORT_HTML'"
  assert "HTML: compliance table in HTML"    "grep -qi 'compliance\\|score\\|grade' '$REPORT_HTML'"
  assert "HTML: findings table in HTML"      "grep -qi 'finding\\|control' '$REPORT_HTML'"
fi

echo ""
echo "═══ Results: ${PASS} passed, ${FAIL} failed ═══"
[[ $FAIL -eq 0 ]]
