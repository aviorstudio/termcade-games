SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c
.DEFAULT_GOAL := help
.NOTPARALLEL:
export GOWORK := off
.PHONY: help install lint test build check dev stop clean
help:
	@echo 'make check: every game test, release policy and WASM package validation'
install:
	mise trust .mise.toml
	mise install go python actionlint shellcheck
	mise exec -- go mod download
lint:
	mise exec -- go vet ./...
	mise exec -- python3 scripts/check-release-workflow.py .github/workflows/release.yml
	mise exec -- actionlint
	mise exec -- shellcheck scripts/build.sh
test:
	mise exec -- go test ./...
	mise exec -- python3 scripts/test-recover-catalog.py
build:
	mise exec -- bash scripts/build.sh
check: lint test build
dev stop:
	@echo '$@: unsupported: WASM game packages run through caller-owned Termcade sessions'
clean:
	mise exec -- python3 -c 'from pathlib import Path; import shutil; [shutil.rmtree(p, ignore_errors=True) for p in Path(".").glob("*/build")]; shutil.rmtree(".artifacts", ignore_errors=True)'
