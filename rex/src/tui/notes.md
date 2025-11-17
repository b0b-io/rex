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

## Current Implementation State

### Phase 1: Foundation (Completed)

The TUI foundation has been implemented following the incremental plan in `tui-implementation-plan.md`. Key components completed:

**Terminal Management**:
- Terminal setup/teardown with alternate screen and raw mode
- Proper cleanup on exit (including Ctrl+C)
- Basic event polling loop

**Theme System** (theme.rs):
- Catppuccin-inspired color scheme (Mocha for dark, Latte for light)
- Semantic color categories (success, warning, error, info, muted)
- Style helper methods to maintain consistency
- Decision: Use RGB colors for precise control vs terminal color indexes
  - Trade-off: Not dependent on user terminal themes, but requires true color support
  - Rationale: Modern terminals support true color, and we want consistent appearance

**Shell Layout System** (shell.rs):
- Five-part layout: title bar, context bar (optional), content, status line (optional), footer
- Dynamic layout calculation based on terminal size and component visibility
- Decision: Content area always fills remaining space using `Constraint::Min(0)`
  - Rationale: Ensures scrollable content has maximum available space

**Shell Components**:
- `TitleBar`: App name on left, registry info on right with `[r]` shortcut
  - Handles narrow terminals by truncating intelligently
- `Footer`: Action list with key bindings, styled differently for enabled/disabled actions
  - Keys highlighted in info color for discoverability
- `Action`: Struct representing key bindings with enabled/disabled state

**Event System** (events.rs):
- Event enum with categories: navigation, actions, special keys, system events
- EventHandler struct for key mapping with configurable vim mode
- Decision: Map crossterm events to high-level application events
  - Rationale: Decouples input handling from view logic, making views testable
  - Trade-off: Extra abstraction layer, but improves testability and maintainability
- Decision: Vim mode as optional parameter
  - Rationale: Power users can use hjkl, but doesn't interfere with arrow keys
  - Implementation: Conditional key mapping based on vim_mode flag
- Ctrl+C maps to Quit event for consistency
- Unknown keys map to Char('\0') to avoid polluting event stream

### Phase 2: Core Infrastructure (In Progress)

**Application State** (app.rs):
- View enum representing all possible application views
- Message enum for worker-to-UI communication
- App struct with state management and view stack
- Decision: Use channels (mpsc) for message passing
  - Rationale: Standard library solution, no external dependencies needed
  - Trade-off: Single producer patterns easier, but sufficient for our needs
- Decision: Result type uses `Send + Sync` error bounds
  - Rationale: Allows errors to be passed across thread boundaries in messages
  - Implementation: `Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>`
- View stack for back navigation (push/pop)
- Event routing delegates to view-specific handlers (to be implemented)
- Worker spawning infrastructure ready for background I/O

**Worker System** (worker.rs):
- Three worker functions: fetch_repositories, fetch_tags, fetch_manifest
- Each worker runs in a background thread, performs I/O, sends result via channel
- Decision: Workers are short-lived, single-purpose functions
  - Rationale: Simple model, automatic resource cleanup
  - Thread spawns, does one task, sends message, exits
- Integration with librex Rex client for registry operations
- Decision: Use blocking librex API (not async)
  - Rationale: Matches TUI synchronous architecture, workers run in threads
  - Trade-off: One thread per operation vs async tasks, but simpler
- Proper error handling with Result propagation
- Workers require mutable Rex instance (librex API design)

### Phase 3: Basic Views (In Progress)

**Repository List View** (views/repos.rs):
- Data Model (Task 3.1) - **Completed**
  - `RepositoryItem`: Struct representing a repository with name, tag count, size, and last updated
  - `RepositoryListState`: State management for the repository list view
  - Navigation: select_next(), select_previous()
  - Selection: selected_item() to get currently selected repository
  - Filtering: filtered_items() for search functionality
  - Decision: Case-sensitive filtering for precision
    - Rationale: Users typically know exact names, case-sensitive is more predictable
    - Trade-off: Less forgiving but more precise
  - Testing: 10 tests covering navigation, selection bounds, and filtering
  - Coverage: All functionality tested including edge cases (empty list, boundaries)

**Next Steps**:
- Task 3.2: Repository list rendering (Table widget with 4 columns)
- Task 3.3: Repository list integration with workers
- Tasks 3.4-3.7: Tag list and image details views

### Pending: Phase 3-5

**Phase 2** will add application state management with message passing for background operations.

**Phase 3** will implement the core views (repository list, tag list, details).

**Phase 4** will add interactive features (search, modals, status banners).

**Phase 5** focuses on polish, error handling, and performance optimization.

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
