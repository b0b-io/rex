# Error Module Notes

## Overview

This document contains implementation notes, decisions, and rationale for the `error` module in `librex`.

## Implementation Decisions

1.  **Refactoring to `thiserror`**:
    - **Initial Implementation**: The module was first built by manually implementing the `std::error::Error` and `std::fmt::Display` traits. This was done to establish a solid, dependency-free foundation.
    - **Rationale for Change**: We refactored to use the `thiserror` crate to reduce boilerplate code, improve readability with the `#[error("...")]` macro, and align with modern Rust error handling idioms. This decision follows the principle of using well-established community crates to improve development velocity and maintainability.

2.  **Error Chaining with `#[source]`**:
    - The `#[source]` attribute is used on variants like `Network`, `Config`, and `Validation`.
    - **Rationale**: These variants are the most likely to wrap underlying errors from external sources (e.g., I/O errors when reading a config, parsing errors for validation, or connection errors from an HTTP client). Using `#[source]` provides a clean way to implement error chaining, which is essential for debugging the root cause of a problem.

3.  **Enum Structure**:
    - The variants of the `RexError` enum are designed to map directly to the error categories defined in `docs/design.md`. This ensures that the implementation stays aligned with the architectural plan.

## Future Considerations

- As new dependencies are added (e.g., an HTTP client, a file loader), other error variants may also need a `source` field to properly chain new error types.
