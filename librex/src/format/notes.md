# Format Module Notes

## Implementation Decisions

1.  **Use `humansize` Crate for Byte Sizes**:
    - **Rationale**: To provide conventional, human-readable file sizes (e.g., "1.2 MB"), we are using the `humansize` crate. This avoids implementing the complex logic for decimal vs. binary units and ensures the output is what users typically expect.

2.  **Use `chrono` and `chrono-humanize` for Timestamps**:
    - **Rationale**: For formatting timestamps into relative strings (e.g., "2 hours ago"), we are using the `chrono` crate as the foundation for time handling, paired with `chrono-humanize`. This combination is the de facto standard in the Rust ecosystem for this task and provides robust, localized, and well-tested functionality.
