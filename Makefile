# Rex - Makefile wrapper for just commands
# This Makefile provides a familiar interface for users who prefer make over just.
# All commands delegate to the justfile for actual implementation.
# The 'just' command runner is automatically installed if not present.

.DEFAULT_GOAL := build

.PHONY: help setup-toolchain setup-dev-tools docs docs-fix build build-release test test-verbose \
        coverage coverage-summary lint lint-fix fmt fmt-check check clean run install stats \
        install-just

# Default target - show help
help:
	@echo "Rex - Makefile wrapper for just commands"
	@echo ""
	@echo "Available targets:"
	@echo "  make                   - Build the project (default)"
	@echo "  make help              - Show this help message"
	@echo ""
	@echo "Setup:"
	@echo "  make setup-toolchain   - Install Rust toolchain using rustup"
	@echo "  make setup-dev-tools   - Install cargo dev tools (tokei, cargo-llvm-cov)"
	@echo "  make install-just      - Install 'just' command runner"
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
	@echo "  make docs              - Lint documentation (optional: needs markdownlint-cli2)"
	@echo "  make docs-fix          - Lint and auto-fix documentation"
	@echo ""
	@echo "Coverage & Stats:"
	@echo "  make coverage          - Generate code coverage report (needs cargo-llvm-cov)"
	@echo "  make coverage-summary  - Show coverage summary (needs cargo-llvm-cov)"
	@echo "  make stats             - Show project statistics (needs tokei)"
	@echo ""
	@echo "Other:"
	@echo "  make install           - Install binary locally"
	@echo ""
	@echo "Notes:"
	@echo "  • Cargo/rustc will be automatically installed if not present"
	@echo "  • Run 'make setup-dev-tools' to install all cargo-based tools"
	@echo "  • Markdown linting requires: npm install -g markdownlint-cli2"
	@echo ""

# Install just command runner if not already installed
install-just:
	@if ! command -v just >/dev/null 2>&1; then \
		echo "📦 Installing 'just' command runner..."; \
		if command -v cargo >/dev/null 2>&1; then \
			cargo install just; \
			echo "✓ 'just' installed successfully"; \
		else \
			echo "❌ Error: cargo not found. Please install Rust first:"; \
			echo "   Run: make setup-toolchain"; \
			exit 1; \
		fi \
	else \
		echo "✓ 'just' is already installed"; \
	fi

# Internal target to ensure just is available
# This is a dependency for all commands that need just
.ensure-just:
	@if ! command -v just >/dev/null 2>&1; then \
		echo "⚠️  'just' command not found. Installing..."; \
		if command -v cargo >/dev/null 2>&1; then \
			cargo install just; \
		else \
			echo ""; \
			echo "❌ Error: 'just' is not installed and cargo is not available."; \
			echo ""; \
			echo "To fix this, either:"; \
			echo "  1. Install Rust toolchain first: make setup-toolchain"; \
			echo "  2. Or install just manually: cargo install just"; \
			echo ""; \
			exit 1; \
		fi \
	fi

# Toolchain setup (doesn't need just)
setup-toolchain:
	@just setup-toolchain || $(MAKE) install-just && just setup-toolchain

# Development tools setup
setup-dev-tools: .ensure-just
	@just setup-dev-tools

# Documentation
docs: .ensure-just
	@just docs

docs-fix: .ensure-just
	@just docs fix

# Build commands
build: .ensure-just
	@just build

build-release: .ensure-just
	@just build-release

# Test commands
test: .ensure-just
	@just test

test-verbose: .ensure-just
	@just test-verbose

# Coverage
coverage: .ensure-just
	@just coverage

coverage-summary: .ensure-just
	@just coverage-summary

# Linting
lint: .ensure-just
	@just lint

lint-fix: .ensure-just
	@just lint fix

# Formatting
fmt: .ensure-just
	@just fmt

fmt-check: .ensure-just
	@just fmt check

# Combined checks
check: .ensure-just
	@just check

# Clean
clean: .ensure-just
	@just clean

# Run with arguments
# Usage: make run ARGS="image tags alpine"
run: .ensure-just
	@just run $(ARGS)

# Install
install: .ensure-just
	@just install

# Statistics
stats: .ensure-just
	@just stats
