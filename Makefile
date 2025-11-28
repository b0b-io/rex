# Rex - Makefile wrapper for just commands
# This Makefile provides a familiar interface for users who prefer make over just.
# All commands delegate to the justfile for actual implementation.

.PHONY: help setup-toolchain docs docs-fix build build-release test test-verbose \
        coverage coverage-summary lint lint-fix fmt fmt-check check clean run install stats

# Default target - show help
help:
	@echo "Rex - Makefile wrapper for just commands"
	@echo ""
	@echo "Available targets:"
	@echo "  make help              - Show this help message"
	@echo "  make setup-toolchain   - Install Rust toolchain using rustup"
	@echo ""
	@echo "Development:"
	@echo "  make build             - Build the project in debug mode"
	@echo "  make build-release     - Build the project in release mode"
	@echo "  make test              - Run tests"
	@echo "  make test-verbose      - Run tests with output"
	@echo "  make run ARGS='...'    - Run the CLI with arguments"
	@echo "  make clean             - Clean build artifacts"
	@echo ""
	@echo "Code Quality:"
	@echo "  make check             - Run all checks (docs, fmt-check, lint)"
	@echo "  make lint              - Run clippy for linting"
	@echo "  make lint-fix          - Run clippy and auto-fix issues"
	@echo "  make fmt               - Format code with rustfmt"
	@echo "  make fmt-check         - Check formatting without changes"
	@echo "  make docs              - Lint documentation files"
	@echo "  make docs-fix          - Lint and auto-fix documentation"
	@echo ""
	@echo "Coverage:"
	@echo "  make coverage          - Generate code coverage report"
	@echo "  make coverage-summary  - Show coverage summary"
	@echo ""
	@echo "Other:"
	@echo "  make install           - Install binary locally"
	@echo "  make stats             - Show project statistics"
	@echo ""
	@echo "Note: This Makefile wraps 'just' commands. Install just:"
	@echo "  cargo install just"
	@echo ""

# Toolchain setup
setup-toolchain:
	@just setup-toolchain

# Documentation
docs:
	@just docs

docs-fix:
	@just docs fix

# Build commands
build:
	@just build

build-release:
	@just build-release

# Test commands
test:
	@just test

test-verbose:
	@just test-verbose

# Coverage
coverage:
	@just coverage

coverage-summary:
	@just coverage-summary

# Linting
lint:
	@just lint

lint-fix:
	@just lint fix

# Formatting
fmt:
	@just fmt

fmt-check:
	@just fmt check

# Combined checks
check:
	@just check

# Clean
clean:
	@just clean

# Run with arguments
# Usage: make run ARGS="image tags alpine"
run:
	@just run $(ARGS)

# Install
install:
	@just install

# Statistics
stats:
	@just stats
