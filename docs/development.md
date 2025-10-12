# Development Guidelines

This document outlines the development methodology and practices for Rex.

## Core Principles

### 1. Test-Driven Development (TDD)

Write tests and code together, not separately:

- ✅ Write a test for the functionality
- ✅ Write the minimal code to make it pass
- ✅ Refactor if needed
- ✅ Ensure all checks pass before moving forward

**Never write code without corresponding tests.**

### 2. Incremental Development

Build one piece at a time:

- ✅ Implement one structure/function at a time
- ✅ Write tests for that specific piece
- ✅ Run `just check` to verify all checks pass
- ✅ Only then move to the next piece

**Do not implement multiple structures or functions before testing.**

### 3. Dependency Order

Follow the dependency graph strictly:

```text
error → digest, reference, format
    → oci → client, auth
        → config, cache, registry → search
            → public API → CLI
```

**Never start a module that depends on unfinished modules.**

### Module Structure

Each module is organized as a directory with separate test files:

```text
librex/src/
├── error/
│   ├── mod.rs            # Module public API
│   ├── tests.rs          # Tests for mod.rs
│   ├── network.rs        # Network error implementations
│   ├── network_tests.rs  # Tests for network.rs
│   ├── auth.rs           # Authentication error implementations
│   └── auth_tests.rs     # Tests for auth.rs
├── digest/
│   ├── mod.rs
│   ├── tests.rs
│   ├── sha256.rs
│   ├── sha256_tests.rs
│   ├── validator.rs
│   └── validator_tests.rs
└── ...
```

**Test File Naming Convention:**

- `mod.rs` → `tests.rs`
- `filename.rs` → `filename_tests.rs`

**Rules:**

- Each module is a directory with a `mod.rs` file.
- Each module should contain a `notes.md` file to document key implementation
  decisions, trade-offs, and rationale.
- Split large modules into multiple files (one concept per file).
- Each implementation file gets a corresponding `*_tests.rs` file.
- Keep files focused and under 500 lines when possible.

### 4. Bisectable Commits

Every commit must be:

- ✅ **Buildable**: `just build` succeeds
- ✅ **Testable**: `just test` passes
- ✅ **Lintable**: `just check` succeeds
- ✅ **Functional**: Adds a complete, usable piece of functionality

**The repository should be in a working state at every commit.**

### 5. Minimal Dependencies

Add dependencies only when needed:

- ❌ Do not add dependencies upfront
- ✅ Add dependencies when implementing the module that uses them
- ✅ Specify exact feature flags needed
- ✅ Document why the dependency is needed

## Commit Guidelines

### Commit Structure

```text
<emoji> <type>: <short summary> 

<detailed description with emojis for visual organization>

• Key change 1
• Key change 2
• Key change 3

Technical details:
  - Implementation detail 1
  - Implementation detail 2

Testing:
  - Test description

Signed-off-by: Your Name <email@example.com>
```

### Commit Types

- ✨ `feat:` - New feature or module
- 🐛 `fix:` - Bug fix
- 🧪 `test:` - Adding or updating tests
- 🔨 `refactor:` - Code refactoring
- 📚 `docs:` - Documentation changes
- 🔧 `chore:` - Build process, tooling, dependencies

### Example Commit

```text
✨ feat: implement error module foundation

Added core error types and categories for Rex error handling:

✅ Core Error Type
   • RexError enum with thiserror derives
   • Implements Display, Debug, Error traits
   • Context preservation through error chain

📦 Error Categories (from design doc)
   • NetworkError - connection, timeout, DNS failures
   • AuthenticationError - 401, 403, token issues
   • ResourceError - 404 not found
   • ConfigError - invalid config, parse errors

🧪 Testing
   • Unit tests for error construction
   • Error display formatting tests
   • Error conversion tests

Dependencies added:
  - thiserror 1.0 - for deriving Error trait

All checks pass: ✓ fmt ✓ clippy ✓ test

Signed-off-by: Your Name <email@example.com>
```

## Development Workflow

### For Each Module

1. **Plan**
   - Review design document section
   - Identify structures and functions needed
   - Determine dependencies required

2. **Implement Incrementally**

   ```bash
   # For each structure/function:
   # 1. Write the test
   # 2. Write the implementation
   # 3. Run checks
   just test          # Run specific tests
   just fmt           # Format code
   just check            # Run all checks
   ```

3. **Commit**

   ```bash
   # When functionality is complete and all checks pass
   git add .
   git commit          # Write descriptive commit message
   ```

4. **Verify**

   ```bash
   # Ensure commit is bisectable
   git checkout HEAD~1
   just ci
   git checkout -
   ```

### Module Completion Checklist

Before marking a module as complete:

- [ ] All planned structures implemented
- [ ] All public functions implemented
- [ ] Unit tests written and passing
- [ ] Documentation comments added
- [ ] Examples in doc comments work
- [ ] Integration with dependent modules tested
- [ ] `just check` passes completely
- [ ] Module exported from lib.rs
- [ ] Commit made with complete summary

## Code Standards

### Rust Edition

Use **edition = "2024"** as specified in project Cargo.toml.

### Code Style

- Follow Rust standard conventions
- Run `just fmt` before every commit
- Ensure `just lint` passes with zero warnings
- Use `#![warn(clippy::all)]` in lib.rs

### Documentation

Every public item must have:

- Doc comment explaining what it does
- Example in doc comment (when appropriate)
- `# Errors` section (for functions that return Result)
- `# Panics` section (for functions that may panic)

Example:

```rust
/// Validates a content digest string.
///
/// # Arguments
///
/// * `digest` - A digest string in format "algorithm:hex"
///
/// # Errors
///
/// Returns `DigestError` if the digest format is invalid or
/// the hex encoding is malformed.
///
/// # Examples
///
/// ```
/// use librex::digest::validate_digest;
///
/// let digest = "sha256:abcd1234...";
/// assert!(validate_digest(digest).is_ok());
/// ```
pub fn validate_digest(digest: &str) -> Result<(), DigestError> {
    // implementation
}
```

### Testing

- Write tests in separate `*_tests.rs` files
- Use descriptive test names: `test_function_name_condition_expected_result`
- Test happy path and error cases
- Use `assert!`, `assert_eq!`, `assert!(matches!(...))`

Example (`validator_tests.rs`):

```rust
use super::*;

#[test]
fn test_validate_digest_valid_sha256_returns_ok() {
    let digest = "sha256:abc123";
    assert!(validate_digest(digest).is_ok());
}

#[test]
fn test_validate_digest_invalid_format_returns_error() {
    let digest = "invalid";
    assert!(validate_digest(digest).is_err());
}
```

## Quality Gates

Every change must pass these gates:

```bash
just check    # Runs: docs, fmt-check, lint, test
```

This ensures:

- ✅ Documentation is lint-free
- ✅ Code is properly formatted
- ✅ No clippy warnings
- ✅ All tests pass

**Do not commit if `just check` fails.**

## Module Development Order

1. error - Error types (no dependencies)
2. digest - Content digest validation (depends: error)
3. reference - Image reference parsing (depends: error)
4. format - Formatting utilities (depends: error)
5. oci - OCI data structures (depends: error, digest, format)
6. client - HTTP client (depends: error, oci)
7. auth - Authentication (depends: error, client)
8. config - Configuration (depends: error)
9. cache - Caching layer (depends: error, oci)
10. registry - Registry operations (depends: all above)
11. search - Search/filter (depends: error, registry)
12. Public API - High-level facade (depends: all modules)
13. CLI - Command-line interface (depends: librex)

## Summary

- 🧪 **Tests first** - Write tests with code, not after
- 📦 **One piece at a time** - Implement incrementally
- ✅ **All checks pass** - Run `just check` before committing
- ✅ **Run tests** - Run `just test` before committing
- 🔄 **Bisectable commits** - Every commit should build and test
- 📝 **Descriptive messages** - Use colorful commit summaries
- 🎯 **Minimal dependencies** - Add only what's needed, when needed
- 📚 **Document everything** - Public items need doc comments
- 🔍 **Follow the plan** - Respect dependency order

---

**Remember**: Quality over speed. A well-tested, incremental approach is faster
in the long run than rushing ahead and debugging later.
