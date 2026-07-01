#!/usr/bin/env bash
# test_all.sh — QXScan Master Test Runner
# Orchestrates all test suites in the demo/ directory:
#   1. test_docker.sh      — Docker health + content assertions (requires Docker + binary)
#   2. classification_test.sh — Classification suite (requires built binary + internet)
#
# Usage:
#   cd demo && bash test_all.sh              # run all suites
#   cd demo && bash test_all.sh --skip-docker # skip Docker service checks
#   cd demo && bash test_all.sh --suite classification --suite edge  # specific suites
#
# Environment variables:
#   QXSCAN_BIN        — path to qxscan binary (default: ../target/release/qxscan)
#   QXSCAN_TIMEOUT    — timeout per classification target in seconds (default: 5)
#   SKIP_DOCKER       — set to 1 to skip Docker-dependent suites

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$SCRIPT_DIR"

PASS_TOTAL=0
FAIL_TOTAL=0
SUITES_RUN=()
SUITES_SKIPPED=()
SUITES_FAILED=()

# ── Argument parsing ────────────────────────────────────────────────────────

SUITES=()
SKIP_DOCKER="${SKIP_DOCKER:-0}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-docker) SKIP_DOCKER=1 ;;
        --suite)
            shift
            SUITES+=("$1")
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: bash test_all.sh [--skip-docker] [--suite <name> ...]"
            echo "  Suites: docker, content, classification, standard, services, edge, all"
            exit 1
            ;;
    esac
    shift
done

# Default: run all suites
if [ ${#SUITES[@]} -eq 0 ]; then
    SUITES=("docker" "content" "classification")
fi

# ── Prerequisites check ─────────────────────────────────────────────────────

QXSCAN_BIN="${QXSCAN_BIN:-$PROJECT_DIR/target/release/qxscan}"
export QXSCAN_BIN

echo "═══════════════════════════════════════════════════════════════"
echo "  QXScan Master Test Runner"
echo "  Binary:      $QXSCAN_BIN"
echo "  Suites:      ${SUITES[*]}"
echo "  Skip Docker: $SKIP_DOCKER"
echo "  Started:     $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# ── Suite runners ───────────────────────────────────────────────────────────

run_suite() {
    local name="$1"
    local cmd="$2"
    local docker_required="${3:-true}"

    if [ "$docker_required" = "true" ] && [ "$SKIP_DOCKER" = "1" ]; then
        echo "⏭️  SKIP: $name (Docker required, --skip-docker active)"
        SUITES_SKIPPED+=("$name")
        return 0
    fi

    echo ""
    echo "─── Suite: $name ───────────────────────────────────────────"
    echo ""

    if eval "$cmd"; then
        echo ""
        echo "✅ Suite PASS: $name"
        SUITES_RUN+=("$name")
    else
        echo ""
        echo "❌ Suite FAIL: $name"
        SUITES_FAILED+=("$name")
        # Don't exit — continue with remaining suites
    fi
}

# ── Execute suites ──────────────────────────────────────────────────────────

for suite in "${SUITES[@]}"; do
    case "$suite" in
        docker)
            if [ ! -f "$QXSCAN_BIN" ]; then
                echo "⏭️  SKIP: Docker Health + Content Assertions (binary not found at $QXSCAN_BIN)"
                echo "   Build it first: cargo build --release"
                SUITES_SKIPPED+=("docker")
                continue
            fi
            run_suite "Docker Health + Content Assertions" \
                "bash '$SCRIPT_DIR/test_docker.sh'" \
                "true"
            ;;
        content)
            # test_docker.sh --assertions-only requires a built binary
            if [ ! -f "$QXSCAN_BIN" ]; then
                echo "⏭️  SKIP: Content Assertions (binary not found at $QXSCAN_BIN)"
                echo "   Build it first: cargo build --release"
                SUITES_SKIPPED+=("content")
                continue
            fi
            run_suite "Content Assertions" \
                "bash '$SCRIPT_DIR/test_docker.sh' --assertions-only" \
                "false"
            ;;
        classification|standard)
            if [ ! -f "$QXSCAN_BIN" ]; then
                echo "⏭️  SKIP: Classification: Standard (binary not found)"
                SUITES_SKIPPED+=("classification-standard")
                continue
            fi
            run_suite "Classification: Standard" \
                "bash '$SCRIPT_DIR/classification_test.sh' standard" \
                "false"
            ;;
        services)
            if [ ! -f "$QXSCAN_BIN" ]; then
                echo "⏭️  SKIP: Classification: Services (binary not found)"
                SUITES_SKIPPED+=("classification-services")
                continue
            fi
            run_suite "Classification: Services" \
                "bash '$SCRIPT_DIR/classification_test.sh' services" \
                "false"
            ;;
        edge)
            if [ ! -f "$QXSCAN_BIN" ]; then
                echo "⏭️  SKIP: Classification: Edge Cases (binary not found)"
                SUITES_SKIPPED+=("classification-edge")
                continue
            fi
            run_suite "Classification: Edge Cases" \
                "bash '$SCRIPT_DIR/classification_test.sh' edge" \
                "false"
            ;;
        all-classification)
            if [ ! -f "$QXSCAN_BIN" ]; then
                echo "⏭️  SKIP: Classification: All (binary not found)"
                SUITES_SKIPPED+=("classification-all")
                continue
            fi
            run_suite "Classification: All" \
                "bash '$SCRIPT_DIR/classification_test.sh' all" \
                "false"
            ;;
        *)
            echo "⚠️  Unknown suite: $suite"
            echo "   Valid: docker, content, classification, standard, services, edge, all-classification, all"
            ;;
    esac
done

# ── Final tally ─────────────────────────────────────────────────────────────

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Test Run Complete"
echo "  $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo ""
echo "  Suites Passed:    ${#SUITES_RUN[@]}"
for s in "${SUITES_RUN[@]}"; do
    echo "    ✅ $s"
done
echo ""
if [ ${#SUITES_FAILED[@]} -gt 0 ]; then
    echo "  Suites Failed:    ${#SUITES_FAILED[@]}"
    for s in "${SUITES_FAILED[@]}"; do
        echo "    ❌ $s"
    done
    echo ""
fi
if [ ${#SUITES_SKIPPED[@]} -gt 0 ]; then
    echo "  Suites Skipped:   ${#SUITES_SKIPPED[@]}"
    for s in "${SUITES_SKIPPED[@]}"; do
        echo "    ⏭️  $s"
    done
    echo ""
fi
echo "═══════════════════════════════════════════════════════════════"

[ ${#SUITES_FAILED[@]} -eq 0 ]
