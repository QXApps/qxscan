#!/usr/bin/env bash
# scan.sh — Run qxscan against all demo services
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
COMPOSE_CMD="${COMPOSE_CMD:-docker compose -f ${SCRIPT_DIR}/../docker-compose.yml}"
QXSCAN="${QXSCAN:-${COMPOSE_CMD} run --rm scanner}"

echo "═══ QXScan Demo — Scanning All Services ═══"
echo ""

echo "--- Scanning Frontend PQC (TLS 1.3 + PQC) ---"
${QXSCAN} scan frontend-pqc --port 443 --no-verify --timeout 15 --standards pqc,pci-dss --output terminal

echo ""
echo "--- Scanning Caddy PQC (dedicated PQC fixture) ---"
${QXSCAN} scan caddy-pqc --port 443 --no-verify --timeout 15 --standards pqc,pci-dss --output terminal

echo ""
echo "--- Scanning Frontend Legacy (TLS 1.2, no PQC) ---"
${QXSCAN} scan frontend-legacy --port 443 --no-verify --timeout 15 --standards pqc,pci-dss --output terminal

echo ""
echo "--- Scanning via ALB ---"
${QXSCAN} scan alb --port 443 --no-verify --timeout 15 --standards pqc,pci-dss --output terminal

echo ""
echo "--- Scanning Backend API ---"
${QXSCAN} scan backend --port 443 --no-verify --timeout 15 --standards pci-dss,hipaa --output terminal

echo ""
echo "--- Scanning Database Proxy (direct TLS) ---"
${QXSCAN} scan db-proxy --port 5433 --no-verify --timeout 15 --standards pci-dss --output terminal

echo ""
echo "--- Scanning Mail (SMTP STARTTLS) ---"
${QXSCAN} scan mail --port 587 --no-verify --timeout 15 --output terminal

echo ""
echo "═══ All scans complete ═══"
