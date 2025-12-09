# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rex is a terminal-native container registry explorer for OCI-compliant registries. It provides both a CLI for scripting/automation and an interactive TUI for visual exploration. The project targets Zot registry primarily but supports any OCI Distribution Specification v1.0+ compliant registry.

## Architecture

### Workspace Structure

Rex uses a Cargo workspace with two crates:

- **`librex/`** - Core library crate providing all registry interaction functionality (UI-agnostic)
- **`rex/`** - CLI/TUI binary crate that consumes librex

This separation ensures the core engine can be embedded in other projects independently.

### Key Architectural Decisions

**Synchronous/Blocking Model**: Rex uses a synchronous architecture with NO async/await:
- HTTP operations use `reqwest::blocking`
- CLI uses `rayon` for data parallelism when needed
- TUI uses threads + channels for background operations
- Rationale: Simpler API, easier debugging, no async runtime overhead, consumers choose their own concurrency

**Two-Tier Caching**: Memory (L1) + Disk (L2) write-through cache
- L1: In-memory LRU cache (fast, cleared on exit)
- L2: Persistent filesystem cache at `~/.cache/rex/`
- Cache is per-registry with separate directories
- TTL-based expiration for different data types

**Configuration Precedence**: CLI flags > Environment vars > Config file > Defaults
- Resolution happens once at startup in `AppContext`
- Context is passed read-only throughout application
- No shortcuts or global state

## Development Workflow

### Essential Commands

```bash
# Build project (debug mode)
just build

# Build release version
just build-release

# Run all checks (docs, format, lint, test)
just check

# Run tests
just test

# Format code
just fmt

# Check formatting
just fmt check

# Run clippy linter
just lint

# Auto-fix clippy warnings
just lint fix

# Lint documentation
just docs

# Auto-fix documentation issues
just docs fix

# Run development version
just run -- --help

# Clean build artifacts
just clean
```

### Running Tests

```bash
# Run all tests
just test

# Run tests with output
just test-verbose

# Run specific test
cargo test test_name

# Run tests for specific module
cargo test --package librex
cargo test --package rex
```

## Code Organization

### librex Core Modules

The library is organized into focused modules:

- **`error/`** - Error types with detailed context (no dependencies)
- **`digest/`** - Content digest validation (sha256, sha512)
- **`reference/`** - Image reference parsing (`registry/repo:tag` or `repo@digest`)
- **`format/`** - Data formatting (sizes, timestamps)
- **`oci/`** - OCI spec data structures (manifests, configs, descriptors)
- **`client/`** - HTTP client for registry communication
- **`auth/`** - Authentication (Basic, Bearer token, credential store)
- **`config/`** - Configuration management (TOML format)
- **`cache/`** - Two-tier caching implementation
- **`registry/`** - Registry operations (catalog, tags, manifests)
- **`search/`** - Fuzzy search with relevance scoring
- **`rex.rs`** - High-level public API (`Rex` and `RexBuilder`)

Module dependency order (implement bottom-up):
```
error → digest, reference, format
    → oci → client, auth
        → config, cache, registry → search
            → rex (public API)
```

### rex Binary Structure

- **`main.rs`** - CLI argument parsing and command routing (uses clap)
- **`context.rs`** - AppContext with configuration precedence
- **`format/`** - Output formatters (TTY-aware, colors, progress bars)
- **`config/`** - Config management (models, persistence)
- **`commands/`** - Command handlers for CLI operations
- **`image/`** - Image-related types and helpers
- **`tui/`** - Interactive TUI implementation (ratatui + crossterm)

### Important Files

- **`dev.md`** - Development methodology (TDD, incremental development, commit guidelines)
- **`librex/design.md`** - Core library architecture and module design
- **`rex/design.md`** - CLI and TUI interface design
- **`librex/API.md`** - High-level public API documentation
- **`req.md`** - Project requirements (not mentioned in list but may exist)

## Development Standards

### Test-Driven Development

**Write tests and code together, not separately**. The workflow is:
1. Write a test for the functionality
2. Write minimal code to make it pass
3. Refactor if needed
4. Run `just check` before moving forward

Module structure pattern:
```
module_name/
├── mod.rs           # Module public API
├── tests.rs         # Tests for mod.rs
├── feature.rs       # Feature implementation
└── feature_tests.rs # Tests for feature.rs
```

Each module should contain a `notes.md` file documenting key implementation decisions and trade-offs.

### Commit Guidelines

Every commit must be:
- **Buildable**: `just build` succeeds
- **Testable**: `just test` passes
- **Lintable**: `just check` succeeds
- **Functional**: Adds a complete, usable piece of functionality

Commit format:
```
<emoji> <type>: <short summary>

<detailed description>

• Key change 1
• Key change 2

Technical details:
  - Implementation detail

Testing:
  - Test description

Signed-off-by: Name <email>
```

**Note**: Do NOT include Claude Code attribution or co-author credits in commit messages. Commits should only contain the technical changes and author's sign-off.

Types: ✨ `feat:`, 🐛 `fix:`, 🧪 `test:`, 🔨 `refactor:`, 📚 `docs:`, 🔧 `chore:`

### Code Style

- Edition: 2024
- Follow Rust standard conventions
- Run `just fmt` before every commit
- Ensure `just check` passes (zero warnings)
- Document public items with doc comments
- Include `# Errors`, `# Panics` sections where applicable
- Keep files focused and under 500 lines when possible

### Testing Standards

- Test files: `mod.rs` → `tests.rs`, `filename.rs` → `filename_tests.rs`
- Descriptive test names: `test_function_name_condition_expected_result`
- Test happy path and error cases
- Use `assert!`, `assert_eq!`, `assert!(matches!(...))`

## Implementation Notes

### Configuration System

Rex uses AppContext for centralized configuration with explicit precedence:

```rust
// In main.rs - resolve once at startup
let ctx = AppContext::build(color_choice, verbosity);

// Pass read-only context throughout
commands::image::handle_list(&ctx, fmt, quiet, filter, limit);

// Handlers pass to formatters
let formatter = format::create_formatter(&ctx);
formatter.success("Operation completed");
```

Never read environment variables or config files in individual functions - use context.

### Output Formatting

Rex uses trait-based TTY-aware formatting:

- **TtyFormatter**: Colors, spinners, progress bars (when stdout/stderr is terminal)
- **PlainFormatter**: Plain text, no ANSI codes (when piped or redirected)

Detection: `NO_COLOR` env var > TTY detection (checks both stdout and stderr)

```rust
let formatter = format::create_formatter(&ctx);

// Success/error messages
formatter.success("Registry initialized");
formatter.error("Connection failed");

// Progress indicators
let spinner = formatter.spinner("Fetching catalog...");
// ... work ...
formatter.finish_progress(spinner, "Fetched 45 repos");
```

### Cache Implementation

Two-tier write-through cache:

1. Check L1 memory cache (< 1μs)
2. Check L2 disk cache (1-10ms), hydrate L1 if hit
3. Fetch from registry (100-500ms), store in both caches

Per-registry isolation: `~/.cache/rex/{registry_hash}/`

TTL strategy:
- Catalog: 5 minutes (repos added/removed moderately)
- Tags: 5 minutes (tags created/deleted frequently)
- Manifest (by tag): 10 minutes (tags can point to different digests)
- Manifest (by digest): 24 hours (immutable)

### Authentication

Supports Basic and Bearer token authentication. Follows OCI Distribution Spec auth flow:
1. Attempt request without auth
2. Receive 401 with WWW-Authenticate header
3. Parse realm, service, scope
4. Request token from auth service
5. Retry with Bearer token

Credentials stored securely in OS keychain when possible.

## Common Development Tasks

### Adding a New CLI Command

1. Define command in `rex/src/main.rs` Subcommand enum
2. Create handler in appropriate module under `rex/src/commands/`
3. Handler receives `&AppContext` and typed parameters
4. Use formatters from context for output
5. Add tests for the handler
6. Update design docs if command adds new functionality

### Adding a New Registry Operation

1. Implement in `librex/src/registry/` module
2. Add tests in corresponding test file
3. Expose through `Rex` struct in `librex/src/rex.rs`
4. Update `librex/API.md` with new public method
5. Consider cache integration if operation fetches data

### Modifying Configuration Schema

1. Update types in `rex/src/config/mod.rs`
2. Update TOML serialization/deserialization
3. Update precedence logic in `AppContext::build()`
4. Update config documentation in `rex/design.md`
5. Add migration logic if breaking existing configs

### Working with the TUI

TUI uses synchronous architecture with background workers:
- Main thread: Render UI at 60 FPS, handle keyboard events
- Worker threads: Spawn on-demand for registry operations
- Communication: mpsc channels for results
- No async/await - just threads + blocking librex calls

## Build Configuration

Project uses Rust Edition 2024. Key dependencies are added incrementally as needed - check `Cargo.toml` files for current dependency set.

Notable dependencies mentioned in design:
- CLI: `clap` for argument parsing, `clap_complete` for shell completions
- HTTP: `reqwest` with `blocking` feature
- Formats: `serde`, `serde_json`, `toml`
- TUI: `ratatui`, `crossterm`
- Formatting: `indicatif` (progress bars), `owo-colors` (terminal colors)
- Parallelism: `rayon` (CLI data parallelism)

## Documentation Standards

Every public item needs:
- Doc comment explaining what it does
- Example in doc comment (when appropriate)
- `# Errors` section for functions returning Result
- `# Panics` section for functions that may panic

Generate docs: `cargo doc --open`

High-level API is prominently displayed; low-level modules are `#[doc(hidden)]` but still public.

## Important Design Principles

1. **Incremental Development**: Build and test one piece at a time
2. **Minimal Dependencies**: Add dependencies only when implementing the module that needs them
3. **Dependency Order**: Never start a module that depends on unfinished modules
4. **Bisectable Commits**: Repository in working state at every commit
5. **No Over-Engineering**: Only make changes that are directly requested or clearly necessary
6. **Explicit Over Implicit**: Clear data flow, no hidden state or globals
7. **Graceful Degradation**: Cache failures don't break functionality

## Unsafe Code and Dependencies Policy

Rex prioritizes memory safety and aims to avoid unsafe Rust code and dependencies with unsafe code whenever possible.

### Policy on Unsafe Dependencies

**Avoid unsafe dependencies unless absolutely necessary.** When considering adding a dependency:

1. **First Check**: Use `cargo tree` to inspect transitive dependencies for crates containing "unsafe" in their name or known to use extensive unsafe code
2. **Prefer Safe Alternatives**: Always look for pure safe Rust alternatives first
3. **Evaluate Necessity**: Question whether the feature requiring unsafe code is truly essential
4. **User Permission Required**: NEVER add a dependency with unsafe code without explicit user approval

### When Unsafe Code is Unavoidable

If no safe alternative exists and the feature is essential, follow this process:

1. **Ask Permission First**: Present the situation to the user before adding the dependency
2. **Provide Context**: Explain what functionality requires unsafe code
3. **Present Alternatives**: List all alternatives including:
   - Pure safe Rust options (even if limited/immature)
   - Removing the feature entirely
   - Implementing a safe subset of functionality
4. **Document Tradeoffs**: For each option, clearly explain:
   - Security implications
   - Performance differences
   - Feature completeness
   - Maintenance burden
   - Project maturity and maintenance status
5. **Let User Decide**: Allow the user to make the informed decision

### Example Decision Process

```
❌ BAD: Silently add `serde_yaml` which transitively pulls in `unsafe-libyaml`

✅ GOOD:
"I need to add YAML serialization support. Here are the options:

1. Use serde_yaml (UNSAFE - depends on unsafe-libyaml for C bindings)
   - Pros: Full YAML 1.2 support, well-tested, fast
   - Cons: Uses unsafe C bindings, deprecated

2. Use serde-saphyr (SAFE - pure Rust)
   - Pros: Safe Rust, panic-free
   - Cons: Deserialization only, not serialization

3. Remove YAML support entirely
   - Pros: No unsafe dependencies, simpler codebase
   - Cons: Users lose YAML output option
   - Note: JSON and table formats already available

What would you like to do?"
```

### Rationale

- **Security**: Unsafe code bypasses Rust's safety guarantees and can introduce vulnerabilities
- **Reliability**: Bugs in unsafe code can cause undefined behavior, crashes, and data corruption
- **Auditability**: Unsafe code requires manual review and is harder to verify
- **Project Philosophy**: Rex emphasizes safety and correctness; this extends to dependencies

## Project Status

Rex is under active development in early implementation phase. The API and commands are subject to change. Current version: 0.0.1
