# Rex - Just commands

# Default recipe to display help information
default:
    @just --list

# Lint documentation files with markdownlint (use 'just docs fix' to auto-fix)
docs fix="":
    #!/usr/bin/env bash
    if [ "{{fix}}" = "fix" ]; then
        markdownlint-cli2 --fix "docs/**/*.md"
    else
        markdownlint-cli2 "docs/**/*.md"
    fi

# Build the project
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Check code without building
check:
    cargo check

# Run clippy for linting (use 'just lint fix' to auto-fix)
lint fix="":
    #!/usr/bin/env bash
    if [ "{{fix}}" = "fix" ]; then
        cargo clippy --fix --allow-dirty --allow-staged -- -D warnings
    else
        cargo clippy -- -D warnings
    fi

# Format code with rustfmt (use 'just fmt check' to only check)
fmt mode="":
    #!/usr/bin/env bash
    if [ "{{mode}}" = "check" ]; then
        cargo fmt -- --check
    else
        cargo fmt
    fi

# Check formatting without making changes (alias for 'just fmt check')
fmt-check:
    @just fmt check

# Run all checks (docs, fmt-check, lint, test)
ci: docs fmt-check lint test

# Clean build artifacts
clean:
    cargo clean

# Run the CLI in development mode
run *ARGS:
    cargo run -- {{ARGS}}

# Install the binary locally
install:
    cargo install --path .

# Show project statistics
stats:
    @echo "=== Code Statistics ==="
    @tokei
    @echo ""
    @echo "=== Documentation Statistics ==="
    @wc -l docs/*.md
