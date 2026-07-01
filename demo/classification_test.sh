#!/usr/bin/env bash
# classification_test.sh — QXScan Classification Test
# Scans targets from demo/targets/ and reports a summary table
# of how each target was classified.
#
# Usage:
#   cd demo && bash classification_test.sh               # targets-standard.txt
#   cd demo && bash classification_test.sh services       # targets-services.txt
#   cd demo && bash classification_test.sh edge           # targets-edge.txt
#   cd demo && bash classification_test.sh all            # all three sequentially
#
# Prerequisites:
#   - ./target/release/qxscan must be built (run 'cargo build --release')
#   - Or use QXSCAN_BIN env var to point to a custom binary

set -euo pipefail

# ── Configuration ──────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGETS_DIR="$SCRIPT_DIR/targets"

QXSCAN_BIN="${QXSCAN_BIN:-$PROJECT_DIR/target/release/qxscan}"
TIMEOUT="${QXSCAN_TIMEOUT:-5}"       # seconds per target — override with env
VERBOSE="${QXSCAN_VERBOSE:-false}"   # set to 'true' to see individual scan results

# Map suite name to targets file
SUITE="${1:-standard}"
case "$SUITE" in
    standard)
        TARGETS_FILE="$TARGETS_DIR/targets-standard.txt"
        SUITE_NAME="Standard (HTTPS 443)"
        ;;
    services)
        TARGETS_FILE="$TARGETS_DIR/targets-services.txt"
        SUITE_NAME="Services (non-443 TLS)"
        ;;
    edge)
        TARGETS_FILE="$TARGETS_DIR/targets-edge.txt"
        SUITE_NAME="Edge Cases (deterministic)"
        ;;
    all|both)
        echo "═══ Running all target suites ═══"
        STD_EXIT=0; SVC_EXIT=0; EDGE_EXIT=0
        bash "$0" standard || STD_EXIT=$?
        bash "$0" services || SVC_EXIT=$?
        bash "$0" edge || EDGE_EXIT=$?
        # Highest non-zero exit wins
        HIGHEST=$STD_EXIT
        [ "$SVC_EXIT" -gt "$HIGHEST" ] && HIGHEST=$SVC_EXIT
        [ "$EDGE_EXIT" -gt "$HIGHEST" ] && HIGHEST=$EDGE_EXIT
        exit "$HIGHEST"
        ;;
    *)
        echo "Usage: bash $0 [standard|services|edge|all]"
        exit 1
        ;;
esac

OUTPUT_LOG=$(mktemp /tmp/qxscan_classification_XXXXXX.txt)

PASS=0
FAIL=0
WARN=0
NOTLS=0
TIMEOUT_COUNT=0
CONNFAIL=0
UNSUPP=0
ERROR=0
OTHER=0

# ── Prerequisites ───────────────────────────────────────────────────────────

if [ ! -f "$QXSCAN_BIN" ]; then
    echo "❌ qxscan binary not found at: $QXSCAN_BIN"
    echo "   Build it first: cargo build --release"
    echo "   Or set QXSCAN_BIN=/path/to/qxscan"
    exit 1
fi

if [ ! -f "$TARGETS_FILE" ]; then
    echo "❌ Targets file not found at: $TARGETS_FILE"
    exit 1
fi

# Count targets (skip comments and blanks)
TOTAL=$(grep -cvE '^\s*(#|$)' "$TARGETS_FILE")
echo "═══ QXScan Classification Test ═══"
echo "  Suite:       $SUITE_NAME"
echo "  Binary:      $QXSCAN_BIN"
echo "  Targets:     $TOTAL (from $(basename "$TARGETS_FILE"))"
echo "  Timeout:     ${TIMEOUT}s per target"
echo "  Started:     $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo ""

# ── Scan all targets ───────────────────────────────────────────────────────

echo "Scanning $TOTAL targets (this may take a few minutes)..."
echo ""

CURRENT=0
while IFS= read -r line || [ -n "$line" ]; do
    # Trim whitespace
    raw="${line#"${line%%[![:space:]]*}"}"
    raw="${raw%"${raw##*[![:space:]]}"}"

    # Skip blank lines and full-line comments
    [ -z "$raw" ] && continue
    [[ "$raw" == \#* ]] && continue

    # Strip inline comments (everything from first # to end of line)
    raw="${raw%%#*}"
    # Re-trim after stripping comment
    raw="${raw%"${raw##*[![:space:]]}"}"
    [ -z "$raw" ] && continue

    CURRENT=$((CURRENT + 1))

    if [ "$VERBOSE" = "true" ]; then
        echo "[$CURRENT/$TOTAL] $raw"
        $QXSCAN_BIN scan "$raw" --no-verify --timeout "$TIMEOUT" 2>/dev/null
        echo ""
    else
        pct=$((CURRENT * 50 / TOTAL))
        bar=$(printf '%*s' "$pct" '')
        bar="${bar// /#}"
        printf "\r  Progress: [%-50s] %3d/%d" "$bar" "$CURRENT" "$TOTAL"

        result=$($QXSCAN_BIN scan "$raw" --no-verify --timeout "$TIMEOUT" 2>/dev/null | grep -E '^\s+Status:\s+' || echo "  Status:    ⚠️ Unknown")
        echo "$raw | $result" >> "$OUTPUT_LOG"
    fi
done < "$TARGETS_FILE"

echo ""  # newline after progress bar

# ── Parse results ──────────────────────────────────────────────────────────

echo ""
echo "═══ Classification Summary: $SUITE_NAME ═══"
echo ""

printf "%-30s %-10s %-22s %s\n" "Target" "Port" "Status" "Category"
printf "%-30s %-10s %-22s %s\n" "------" "----" "------" "--------"

CATEGORY=""

while IFS= read -r line || [ -n "$line" ]; do
    raw="${line#"${line%%[![:space:]]*}"}"
    raw="${raw%"${raw##*[![:space:]]}"}"

    # Track category changes from comments
    if [[ "$raw" == \#\ ──* ]]; then
        CATEGORY="${raw#*── }"
        CATEGORY="${CATEGORY%% ──*}"
        continue
    fi

    [ -z "$raw" ] && continue
    [[ "$raw" == \#* ]] && continue

    # Strip inline comments (same as first loop) so grep matches the log
    raw="${raw%%#*}"
    # Re-trim after stripping comment
    raw="${raw%"${raw##*[![:space:]]}"}"
    [ -z "$raw" ] && continue

    result_line=$(grep -F "$raw |" "$OUTPUT_LOG" | head -1)

    if [ -z "$result_line" ]; then
        printf "%-30s %-10s %-22s %s\n" "$raw" "-" "⚠️ No data" "$CATEGORY"
        OTHER=$((OTHER + 1))
        continue
    fi

    target_part="${result_line%% \| *}"
    status_part="${result_line#*| }"
    status_text=$(echo "$status_part" | sed 's/.*Status://; s/^ *//; s/ *$//')

    if [[ "$target_part" == *:* ]]; then
        port="${target_part##*:}"
    else
        port="443"
    fi

    host="${target_part%%:*}"

    case "$status_text" in
        *"✅ Pass"*)          PASS=$((PASS + 1)) ;;
        *"❌ Fail"*)          FAIL=$((FAIL + 1)) ;;
        *"⚠️ Warn"*)          WARN=$((WARN + 1)) ;;
        *"ℹ️ No TLS"*)        NOTLS=$((NOTLS + 1)) ;;
        *"⏰ Timeout"*)       TIMEOUT_COUNT=$((TIMEOUT_COUNT + 1)) ;;
        *"❌ Connection Failed"*) CONNFAIL=$((CONNFAIL + 1)) ;;
        *"❌ Unsupported Protocol"*) UNSUPP=$((UNSUPP + 1)) ;;
        *"❌ Error"*)         ERROR=$((ERROR + 1)) ;;
        *)                    OTHER=$((OTHER + 1)) ;;
    esac

    printf "%-30s %-10s %-22s %s\n" "$host" "$port" "$status_text" "$CATEGORY"

done < "$TARGETS_FILE"

# ── Final tally ─────────────────────────────────────────────────────────────

echo ""
echo "─── Results: $SUITE_NAME ────────────────────────────────────────────"
printf "  %-30s %3d\n" "✅ Pass"                    "$PASS"
printf "  %-30s %3d\n" "❌ Fail"                    "$FAIL"
printf "  %-30s %3d\n" "⚠️ Warn"                    "$WARN"
printf "  %-30s %3d\n" "ℹ️ No TLS"                  "$NOTLS"
printf "  %-30s %3d\n" "⏰ Timeout"                 "$TIMEOUT_COUNT"
printf "  %-30s %3d\n" "❌ Connection Failed"       "$CONNFAIL"
printf "  %-30s %3d\n" "❌ Unsupported Protocol"    "$UNSUPP"
printf "  %-30s %3d\n" "❌ Error"                   "$ERROR"
printf "  %-30s %3d\n" "⚠️ Other"                  "$OTHER"
echo "  ─────────────────────────────────────────────"
printf "  %-30s %3d\n" "Total" "$((PASS + FAIL + WARN + NOTLS + TIMEOUT_COUNT + CONNFAIL + UNSUPP + ERROR + OTHER))"
echo ""
echo "  Scanned at: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "  Timeout:    ${TIMEOUT}s per target"
echo "  Full log:   $OUTPUT_LOG"

SUCCESS=$((PASS + FAIL + WARN + NOTLS + UNSUPP))
if [ "$SUCCESS" -eq 0 ]; then
    echo ""
    echo "⚠️  No targets were successfully classified — all returned errors or timed out."
    echo "    Check network connectivity or try with a longer --timeout."
    exit 1
fi

echo ""
echo "✅ $SUITE_NAME — classification test complete."
