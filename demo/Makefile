.PHONY: help build up down status logs scan scan-all pqc-check legacy-check clean rebuild demo-suite test-classification test-edge test-services test-standard test-all

SHELL := /bin/bash

# The compose file lives at project root now
COMPOSE_FILE := $(shell [ -f ../docker-compose.yml ] && echo ../docker-compose.yml || echo docker-compose.yml)
COMPOSE := docker compose -f $(COMPOSE_FILE)
# Makefile's own directory — safe regardless of where make was invoked
SCRIPT_DIR := $(dir $(lastword $(MAKEFILE_LIST)))

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build all demo images
	$(COMPOSE) build

build-scanner: ## Build only the scanner image
	$(COMPOSE) build scanner

up: ## Start core demo stack (full)
	$(COMPOSE) up -d
	@echo "Demo stack started. ALB available at https://localhost:8443"
	@echo "Frontend PQC (direct):  docker compose exec frontend-pqc curl -sk https://localhost | head -5"
	@echo "Frontend Legacy (direct): docker compose exec frontend-legacy curl -sk https://localhost | head -5"

up-alb: ## Start ALB + frontends + backend + db only
	@echo "Starting ALB stack (all core services start by default)..."
	$(COMPOSE) up -d
	@echo "Stack started. ALB available at https://localhost:8443"

down: ## Stop all demo services
	$(COMPOSE) down -v

status: ## Show container status
	$(COMPOSE) ps

logs: ## Show logs from all services
	$(COMPOSE) logs -f

scan: ## Scan a specific demo service (usage: make scan SVC=frontend-pqc PORT=443 STANDARDS=pqc)
	$(COMPOSE) run --rm scanner scan $(SVC) --port $(PORT) --no-verify --timeout 15 --standards $(STANDARDS) --output terminal

scan-all: ## Scan all demo services sequentially
	@echo "=== Scanning all services ==="
	$(COMPOSE) run --rm scanner scan frontend-pqc --port 443 --no-verify --timeout 15 --standards pqc,pci-dss --output terminal
	@echo ""
	$(COMPOSE) run --rm scanner scan caddy-pqc --port 443 --no-verify --timeout 15 --standards pqc,pci-dss --output terminal
	@echo ""
	$(COMPOSE) run --rm scanner scan frontend-legacy --port 443 --no-verify --timeout 15 --standards pqc,pci-dss --output terminal
	@echo ""
	$(COMPOSE) run --rm scanner scan alb --port 443 --no-verify --timeout 15 --standards pqc,pci-dss --output terminal
	@echo ""
	$(COMPOSE) run --rm scanner scan backend --port 443 --no-verify --timeout 15 --standards pci-dss,hipaa --output terminal
	@echo ""
	$(COMPOSE) run --rm scanner scan db-proxy --port 5433 --no-verify --timeout 15 --standards pci-dss --output terminal
	@echo ""
	$(COMPOSE) run --rm scanner scan mail --port 587 --no-verify --timeout 15 --output terminal

pqc-check: ## Run PQC readiness scan against PQC-enabled services
	@echo "=== PQC Readiness Check ==="
	$(COMPOSE) run --rm scanner scan frontend-pqc --port 443 --no-verify --timeout 15 --standards pqc --output terminal
	@echo ""
	$(COMPOSE) run --rm scanner scan caddy-pqc --port 443 --no-verify --timeout 15 --standards pqc --output terminal
	@echo ""
	$(COMPOSE) run --rm scanner scan alb --port 443 --no-verify --timeout 15 --standards pqc --output terminal
	@echo ""
	$(COMPOSE) run --rm scanner scan backend --port 443 --no-verify --timeout 15 --standards pqc --output terminal

legacy-check: ## Verify legacy frontend is detected as non-PQC
	@echo "=== Legacy Detection Check ==="
	$(COMPOSE) run --rm scanner scan frontend-legacy --port 443 --no-verify --timeout 15 --standards pqc --output terminal

export-json: ## Export scan results as JSON
	$(COMPOSE) run --rm scanner scan frontend-pqc --port 443 --no-verify --timeout 15 --output json --report-file /reports/scan.json
export-html: ## Export scan results as HTML
	$(COMPOSE) run --rm scanner scan frontend-pqc --port 443 --no-verify --timeout 15 --output html --report-file /reports/scan.html

demo-suite: ## Run demo integration test (Docker health + content assertions)
	$(COMPOSE) up demo-scan

clean: ## Clean up all data volumes, local images, and stale results
	$(COMPOSE) down -v --rmi local --remove-orphans
	rm -f results/*.json reports/report.html

clean-results: ## Remove stale scan result files only
	rm -f results/*.json reports/*.json reports/*.html

rebuild: clean build up ## Full rebuild and restart

# ── Test Targets ────────────────────────────────────────────────────────────

test-classification: ## Run classification test against standard targets
	cd $(SCRIPT_DIR) && bash classification_test.sh standard

test-services: ## Run classification test against service protocol targets
	cd $(SCRIPT_DIR) && bash classification_test.sh services

test-edge: ## Run classification test against edge-case targets
	cd $(SCRIPT_DIR) && bash classification_test.sh edge

test-all: ## Run all test suites (requires Docker demo stack + built binary)
	cd $(SCRIPT_DIR) && bash test_all.sh

# Default target
.DEFAULT_GOAL := help
