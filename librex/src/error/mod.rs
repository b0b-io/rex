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

impl RexError {
    /// Creates a new network error.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::error::RexError;
    ///
    /// let err = RexError::network("connection refused");
    /// assert!(matches!(err, RexError::Network { .. }));
    /// ```
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a new network error with a source error.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::error::RexError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
    /// let err = RexError::network_with_source("failed to connect", io_err);
    /// assert!(matches!(err, RexError::Network { .. }));
    /// ```
    pub fn network_with_source<S, E>(message: S, source: E) -> Self
    where
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Network {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Creates a new authentication error.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::error::RexError;
    ///
    /// let err = RexError::authentication("invalid credentials", Some(401));
    /// assert!(matches!(err, RexError::Authentication { .. }));
    /// ```
    pub fn authentication<S: Into<String>>(message: S, status_code: Option<u16>) -> Self {
        Self::Authentication {
            message: message.into(),
            status_code,
        }
    }

    /// Creates a new not found error.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::error::RexError;
    ///
    /// let err = RexError::not_found("repository", "myrepo");
    /// assert!(matches!(err, RexError::NotFound { .. }));
    /// ```
    pub fn not_found<S: Into<String>>(resource_type: S, name: S) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            name: name.into(),
        }
    }

    /// Creates a new rate limit error.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::error::RexError;
    ///
    /// let err = RexError::rate_limit("too many requests", Some(60));
    /// assert!(matches!(err, RexError::RateLimit { .. }));
    /// ```
    pub fn rate_limit<S: Into<String>>(message: S, retry_after: Option<u64>) -> Self {
        Self::RateLimit {
            message: message.into(),
            retry_after,
        }
    }

    /// Creates a new server error.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::error::RexError;
    ///
    /// let err = RexError::server("internal server error", 500);
    /// assert!(matches!(err, RexError::Server { .. }));
    /// ```
    pub fn server<S: Into<String>>(message: S, status_code: u16) -> Self {
        Self::Server {
            message: message.into(),
            status_code,
        }
    }

    /// Creates a new validation error.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::error::RexError;
    ///
    /// let err = RexError::validation("invalid manifest format");
    /// assert!(matches!(err, RexError::Validation { .. }));
    /// ```
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a new validation error with a source error.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::error::RexError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::InvalidData, "invalid data");
    /// let err = RexError::validation_with_source("invalid format", io_err);
    /// assert!(matches!(err, RexError::Validation { .. }));
    /// ```
    pub fn validation_with_source<S, E>(message: S, source: E) -> Self
    where
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Validation {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Creates a new configuration error.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::error::RexError;
    ///
    /// let err = RexError::config("invalid config file", Some("/path/to/config.toml"));
    /// assert!(matches!(err, RexError::Config { .. }));
    /// ```
    pub fn config<S: Into<String>>(message: S, path: Option<S>) -> Self {
        Self::Config {
            message: message.into(),
            path: path.map(|p| p.into()),
            source: None,
        }
    }

    /// Creates a new configuration error with a source error.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::error::RexError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let err = RexError::config_with_source("failed to read config", Some("/path/to/config.toml"), io_err);
    /// assert!(matches!(err, RexError::Config { .. }));
    /// ```
    pub fn config_with_source<S, E>(message: S, path: Option<S>, source: E) -> Self
    where
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Config {
            message: message.into(),
            path: path.map(|p| p.into()),
            source: Some(Box::new(source)),
        }
    }
}
