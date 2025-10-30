//! Rex - Container Registry Explorer Library
//!
//! Rex provides a high-level, easy-to-use interface for interacting with
//! OCI-compliant container registries.
//!
//! # Quick Start
//!
//! ```no_run
//! use librex::Rex;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to a registry
//!     let mut rex = Rex::connect("http://localhost:5000")?;
//!
//!     // List all repositories
//!     let repos = rex.list_repositories()?;
//!     for repo in repos {
//!         println!("{}", repo);
//!     }
//!
//!     // Search for repositories
//!     let results = rex.search_repositories("alpine")?;
//!     for result in results {
//!         println!("{}", result.value);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - **Simple API**: High-level [`Rex`] struct for common operations
//! - **Fuzzy Search**: Fast, fzf-like fuzzy matching for repositories and tags
//! - **Caching**: Optional persistent caching for improved performance
//! - **Authentication**: Support for Basic and Bearer token authentication
//! - **OCI Compliant**: Full support for OCI Distribution Specification
//!
//! # Main Types
//!
//! - [`Rex`] - Main entry point for registry operations
//! - [`RexBuilder`] - Builder for advanced configuration
//! - [`SearchResult`] - Search result with relevance scoring
//! - [`Credentials`] - Authentication credentials
//! - [`Reference`] - Image reference parsing and manipulation
//! - [`Digest`] - Content digest validation and handling
//!
//! # Architecture
//!
//! Rex is organized into modules:
//!
//! - **High-level API** ([`rex`]) - Recommended for most users
//! - **Low-level modules** - Available for advanced use cases (hidden from docs)
//!
//! For most use cases, you should use the [`Rex`] struct. The low-level modules
//! are available if you need fine-grained control, but are not shown in the
//! documentation by default.

#![warn(clippy::all)]

/// Returns the librex crate version.
///
/// This is useful for version reporting in CLI tools and debugging.
///
/// # Examples
///
/// ```
/// let version = librex::version();
/// assert!(!version.is_empty());
/// ```
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// High-level public API (main entry point)
mod rex;
pub use rex::{Rex, RexBuilder};

// Re-export commonly used types for convenience
pub use auth::Credentials;
pub use config::Config;
pub use digest::Digest;
pub use error::{Result, RexError};
pub use oci::ManifestOrIndex;
pub use reference::Reference;
pub use search::SearchResult;

// Low-level implementation modules (hidden from docs but still public)
// These are available for advanced users who need fine-grained control
#[doc(hidden)]
pub mod auth;
#[doc(hidden)]
pub mod cache;
#[doc(hidden)]
pub mod client;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod digest;
#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod format;
#[doc(hidden)]
pub mod oci;
#[doc(hidden)]
pub mod reference;
#[doc(hidden)]
pub mod registry;
#[doc(hidden)]
pub mod search;
