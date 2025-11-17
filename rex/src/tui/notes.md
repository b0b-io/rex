# TUI Module - Implementation Notes

## Overview

This module provides the Terminal User Interface (TUI) for Rex, allowing interactive exploration of container registries.

## Design Decisions

### 1. Synchronous Architecture (No async/await)

**Decision**: Use blocking operations with threads instead of async/await.

**Rationale**:
- Matches the rest of Rex codebase (uses `reqwest::blocking`)
- Simpler to reason about for TUI event loops
- Background work handled via `std::thread::spawn` + channels
- Avoids async runtime overhead for TUI operations

**Trade-offs**:
- Pro: Simple threading model, no async complexity
- Pro: Main UI loop is straightforward synchronous code
- Con: Slightly more verbose thread spawning vs async tasks

### 2. Terminal Backend: crossterm

**Decision**: Use crossterm as the terminal backend for ratatui.

**Rationale**:
- Cross-platform (Windows, Linux, macOS)
- Well-maintained and widely used
- Direct integration with ratatui
- Handles raw mode, alternate screen, and events

**Alternatives Considered**:
- termion: Unix-only, doesn't support Windows
- Direct VT100: Too low-level, not worth the effort

### 3. Test Strategy

**Decision**: Use signature verification tests for terminal functions.

**Rationale**:
- Full terminal initialization requires real terminal (not available in tests)
- Verify function signatures compile correctly
- Integration testing done manually or in CI with real terminal

**Trade-offs**:
- Pro: Tests compile and verify types
- Con: Not testing actual terminal behavior
- Mitigation: Manual testing + integration tests

### 4. Error Handling

**Decision**: Use `Box<dyn std::error::Error>` for Result type.

**Rationale**:
- Terminal operations can fail in multiple ways (IO, terminal state, etc.)
- Don't need granular error types at TUI level
- Propagate errors to CLI layer for user-friendly messages

### 5. Incremental Development

**Decision**: Add `#[allow(dead_code)]` during incremental development.

**Rationale**:
- Following TDD approach, building one piece at a time
- Functions exist but aren't yet integrated
- TODO reminder to remove these attributes when functions are called

**Next Steps**:
- Remove `#[allow(dead_code)]` when integrating with CLI
- Add `tui` subcommand to main.rs

## Implementation Status

### ✅ Completed
- [x] Module structure (mod.rs + tests.rs)
- [x] Terminal initialization (setup_terminal)
- [x] Terminal cleanup (restore_terminal)
- [x] Basic event loop (run function)
- [x] Quit on 'q' functionality

### ⏳ Pending
- [ ] Theme system
- [ ] Shell components (title, footer, context bar)
- [ ] Event handling system
- [ ] Application state management
- [ ] Worker threads for background operations
- [ ] Views (repository list, tag list, details)
- [ ] Integration with CLI

## Testing Notes

Current tests verify:
1. Module compiles correctly
2. Function signatures are correct
3. Types match expected interfaces

Future testing will include:
- Mock terminal testing (when available)
- Integration tests with real terminal
- Event handling tests
- State transition tests

## Dependencies

### Direct Dependencies
- `crossterm 0.27` - Terminal control and events
- `ratatui 0.25` - TUI framework and widgets

### Why These Versions?
- Latest stable versions as of implementation
- Both have clean APIs and good documentation
- Mature libraries with active maintenance
