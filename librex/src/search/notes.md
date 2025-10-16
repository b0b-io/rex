# Search Module Notes

## 1. Implementation Decisions

1.  **Use `nucleo-matcher` Crate**:
    - **Rationale**: To provide a high-quality, `fzf`-like fuzzy search experience, we chose to use the `nucleo-matcher` crate. This is the core matching engine from the `nucleo` project (a rewrite of `fzf` in Rust) and is highly optimized for performance. Building our own fuzzy-matching algorithm would be complex and unnecessary. This aligns with our "Don't Reinvent the Wheel" principle.
    - **Implementation**: The `search` module is a stateless collection of pure functions that wrap the `nucleo-matcher` API. It provides a simple, high-level interface for searching lists of strings.

2.  **API Design**:
    - **`fuzzy_search`**: A generic, core function that takes a query, a list of targets, and a `CaseMatching` mode.
    - **Specialized Functions**: `search_repositories`, `search_tags`, and `search_images` provide a more ergonomic API for specific use cases.
    - **`search_images` Logic**: This function handles combined queries (e.g., "repo:tag") by splitting the query, searching repositories first, and then searching the tags of the matching repositories. This provides an intuitive and powerful way for users to narrow down images.

3.  **Scoring and Ranking**:
    - All scoring and ranking logic is delegated to `nucleo-matcher`.
    - Results are wrapped in a `SearchResult` struct to expose both the matched value and its relevance score.
    - The final results are sorted first by score (descending) and then alphabetically as a secondary criterion to ensure a stable and predictable order.
