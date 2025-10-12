use super::*;
use std::error::Error;

#[test]
fn test_network_error_connection_refused() {
    let err = RexError::Network {
        message: "connection refused".to_string(),
        source: None,
    };

    assert!(matches!(err, RexError::Network { .. }));
    assert!(err.to_string().contains("connection refused"));
}

#[test]
fn test_network_error_timeout() {
    let err = RexError::Network {
        message: "request timeout after 30s".to_string(),
        source: None,
    };

    assert!(err.to_string().contains("timeout"));
}

#[test]
fn test_authentication_error_invalid_credentials() {
    let err = RexError::Authentication {
        message: "invalid username or password".to_string(),
        status_code: Some(401),
    };

    assert!(matches!(err, RexError::Authentication { .. }));
}

#[test]
fn test_authentication_error_insufficient_permissions() {
    let err = RexError::Authentication {
        message: "insufficient permissions".to_string(),
        status_code: Some(403),
    };

    assert!(err.to_string().contains("insufficient permissions"));
}

#[test]
fn test_resource_error_not_found() {
    let err = RexError::NotFound {
        resource_type: "repository".to_string(),
        name: "myrepo".to_string(),
    };

    assert!(matches!(err, RexError::NotFound { .. }));
    assert!(err.to_string().contains("repository"));
    assert!(err.to_string().contains("myrepo"));
}

#[test]
fn test_resource_error_tag_not_found() {
    let err = RexError::NotFound {
        resource_type: "tag".to_string(),
        name: "v1.0.0".to_string(),
    };

    assert!(err.to_string().contains("tag"));
    assert!(err.to_string().contains("v1.0.0"));
}

#[test]
fn test_rate_limit_error() {
    let err = RexError::RateLimit {
        message: "too many requests".to_string(),
        retry_after: Some(60),
    };

    assert!(matches!(err, RexError::RateLimit { .. }));
}

#[test]
fn test_server_error_internal() {
    let err = RexError::Server {
        message: "internal server error".to_string(),
        status_code: 500,
    };

    assert!(matches!(err, RexError::Server { .. }));
    assert!(err.to_string().contains("internal server error"));
}

#[test]
fn test_validation_error_invalid_manifest() {
    let err = RexError::Validation {
        message: "invalid manifest format".to_string(),
        source: None,
    };

    assert!(matches!(err, RexError::Validation { .. }));
}

#[test]
fn test_validation_error_digest_mismatch() {
    let err = RexError::Validation {
        message: "digest mismatch".to_string(),
        source: None,
    };

    assert!(err.to_string().contains("digest mismatch"));
}

#[test]
fn test_config_error_invalid_file() {
    let err = RexError::Config {
        message: "invalid config file".to_string(),
        path: Some("/path/to/config.toml".to_string()),
        source: None,
    };

    assert!(matches!(err, RexError::Config { .. }));
    assert!(err.to_string().contains("invalid config file"));
}

#[test]
fn test_error_implements_error_trait() {
    let err = RexError::Network {
        message: "test error".to_string(),
        source: None,
    };

    // Should implement Error trait
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_error_implements_display() {
    let err = RexError::NotFound {
        resource_type: "image".to_string(),
        name: "alpine".to_string(),
    };

    let display_str = format!("{}", err);
    assert!(!display_str.is_empty());
}

#[test]
fn test_error_implements_debug() {
    let err = RexError::Network {
        message: "connection failed".to_string(),
        source: None,
    };

    let debug_str = format!("{:?}", err);
    assert!(!debug_str.is_empty());
}

#[test]
fn test_config_error_with_source() {
    // Create a sample source error
    let source_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");

    let err = RexError::Config {
        message: "failed to read config".to_string(),
        path: Some("/path/to/config.toml".to_string()),
        source: Some(Box::new(source_error)),
    };

    // Check that the source is correctly propagated
    assert!(err.source().is_some());
    assert!(err.source().unwrap().to_string().contains("file not found"));
}

// Tests for helper constructors

#[test]
fn test_network_helper_constructor() {
    let err = RexError::network("connection refused");
    assert!(matches!(err, RexError::Network { .. }));
    assert!(err.to_string().contains("connection refused"));
}

#[test]
fn test_network_with_source_helper_constructor() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
    let err = RexError::network_with_source("failed to connect", io_err);
    assert!(matches!(err, RexError::Network { .. }));
    assert!(err.source().is_some());
}

#[test]
fn test_authentication_helper_constructor() {
    let err = RexError::authentication("invalid credentials", Some(401));
    assert!(matches!(err, RexError::Authentication { .. }));
    assert!(err.to_string().contains("invalid credentials"));
}

#[test]
fn test_not_found_helper_constructor() {
    let err = RexError::not_found("repository", "myrepo");
    assert!(matches!(err, RexError::NotFound { .. }));
    assert!(err.to_string().contains("repository"));
    assert!(err.to_string().contains("myrepo"));
}

#[test]
fn test_rate_limit_helper_constructor() {
    let err = RexError::rate_limit("too many requests", Some(60));
    assert!(matches!(err, RexError::RateLimit { .. }));
    assert!(err.to_string().contains("too many requests"));
}

#[test]
fn test_server_helper_constructor() {
    let err = RexError::server("internal server error", 500);
    assert!(matches!(err, RexError::Server { .. }));
    assert!(err.to_string().contains("internal server error"));
}

#[test]
fn test_validation_helper_constructor() {
    let err = RexError::validation("invalid manifest format");
    assert!(matches!(err, RexError::Validation { .. }));
    assert!(err.to_string().contains("invalid manifest format"));
}

#[test]
fn test_validation_with_source_helper_constructor() {
    let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid data");
    let err = RexError::validation_with_source("invalid format", io_err);
    assert!(matches!(err, RexError::Validation { .. }));
    assert!(err.source().is_some());
}

#[test]
fn test_config_helper_constructor() {
    let err = RexError::config("invalid config file", Some("/path/to/config.toml"));
    assert!(matches!(err, RexError::Config { .. }));
    assert!(err.to_string().contains("invalid config file"));
}

#[test]
fn test_config_with_source_helper_constructor() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = RexError::config_with_source(
        "failed to read config",
        Some("/path/to/config.toml"),
        io_err,
    );
    assert!(matches!(err, RexError::Config { .. }));
    assert!(err.source().is_some());
}
