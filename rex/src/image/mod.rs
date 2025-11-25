//! Image metadata operations.
//!
//! This module provides shared functionality for fetching and formatting image
//! metadata (tags, manifests, configurations) used by both CLI and TUI.

pub mod metadata;
pub mod types;

// Re-export commonly used types
pub use metadata::{RepositoryMetadataFetcher, TagMetadataFetcher};
pub use types::{RepositoryItem, TagInfo};
