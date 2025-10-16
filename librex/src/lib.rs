//! Rex - Container Registry Explorer Library
//!
//! This library provides functionality for interacting with OCI-compliant
//! container registries.

#![warn(clippy::all)]

// Modules will be added incrementally following the dependency order:
// error → digest, reference, format → oci → client, auth → config, cache, registry → search

pub mod auth;
pub mod cache;
pub mod client;
pub mod config;
pub mod digest;
pub mod error;
pub mod format;
pub mod oci;
pub mod reference;
pub mod registry;
pub mod search;
