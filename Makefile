SHELL := /bin/bash

SCAN_TARGET ?= example.com
SCAN_TIMEOUT ?= 10
SCAN_OUTPUT ?= terminal
SCAN_REPORT ?=
SCAN_STANDARDS ?= pci-dss
SCAN_EXTRA ?=

.PHONY: setup build build-release test lint fmt fmt-check check run scan scan-help install clean

setup:
	cargo fetch

build:
	cargo build

build-release:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

check: fmt-check lint test build

run:
	cargo run -- scan $(TARGET)

scan:
	cargo run -- scan "$(SCAN_TARGET)" --timeout "$(SCAN_TIMEOUT)" --output "$(SCAN_OUTPUT)" --standards "$(SCAN_STANDARDS)" $(if $(SCAN_REPORT),--report-file "$(SCAN_REPORT)",) $(SCAN_EXTRA)

scan-help:
	target/debug/qxscan scan --help

install:
	cargo install --path .

clean:
	cargo clean
