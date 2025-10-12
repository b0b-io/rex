//! Error types for Rex
//!
//! This module provides comprehensive error handling for all Rex operations.
//! All errors implement the standard Error trait and provide context-rich
//! error messages.

use thiserror::Error;

#[cfg(test)]
mod tests;

/// Main error type for Rex operations
#[derive(Error, Debug)]
pub enum RexError {
    /// Network-related errors (connection, timeout, DNS)
    #[error("Network error: {message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Authentication errors (401, 403, token issues)
    #[error("Authentication error (status: {status_code:?}): {message}")]
    Authentication {
        message: String,
        status_code: Option<u16>,
    },

    /// Resource not found errors (404)
    #[error("{resource_type} not found: {name}")]
    NotFound { resource_type: String, name: String },

    /// Rate limiting errors (429)
    #[error("Rate limit: {message}")]
    RateLimit {
        message: String,
        retry_after: Option<u64>,
    },

    /// Server errors (500, 503)
    #[error("Server error (status: {status_code}): {message}")]
    Server { message: String, status_code: u16 },

    /// Validation errors (invalid manifest, digest mismatch, etc.)
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration errors (invalid config file, missing settings)
    #[error("Configuration error: {message}")]
    Config {
        message: String,
        path: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Result type alias for Rex operations
pub type Result<T> = std::result::Result<T, RexError>;
