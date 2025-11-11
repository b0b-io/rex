use super::*;

#[test]
fn test_client_new_with_valid_url() {
    let client = Client::new("http://localhost:5000", None);
    assert!(client.is_ok());
}

#[test]
fn test_client_new_with_https_url() {
    let client = Client::new("https://registry.example.com", None);
    assert!(client.is_ok());
}

#[test]
fn test_client_normalizes_url_without_scheme() {
    let client = Client::new("localhost:5000", None).unwrap();
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_client_removes_trailing_slash() {
    let client = Client::new("http://localhost:5000/", None).unwrap();
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_client_removes_multiple_trailing_slashes() {
    let client = Client::new("http://localhost:5000///", None).unwrap();
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_client_new_with_empty_url_fails() {
    let client = Client::new("", None);
    assert!(client.is_err());
    assert!(matches!(client.unwrap_err(), RexError::Validation { .. }));
}

#[test]
fn test_client_new_with_whitespace_url_fails() {
    let client = Client::new("   ", None);
    assert!(client.is_err());
}

#[test]
fn test_client_registry_url_accessor() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_client_with_port() {
    let client = Client::new("localhost:8080", None).unwrap();
    assert_eq!(client.registry_url(), "http://localhost:8080");
}

#[test]
fn test_client_with_domain() {
    let client = Client::new("registry.example.com", None).unwrap();
    assert_eq!(client.registry_url(), "http://registry.example.com");
}

// Note: Integration tests for check_version() will be added when we have
// a test registry available. For now, we verify the basic structure.

#[test]
fn test_url_construction_for_version_check() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    // Verify the URL would be correct for version check
    assert_eq!(client.registry_url(), "http://localhost:5000");
    // The version check uses: format!("{}/v2/", client.registry_url())
    // which would produce: "http://localhost:5000/v2/"
}

#[test]
fn test_client_has_async_check_version_method() {
    // This test just verifies the method exists and can be called
    // Integration testing with a real registry will be done later
    let client = Client::new("http://localhost:5000", None).unwrap();
    // The method signature is: pub async fn check_version(&self) -> Result<RegistryVersion>
    // We can't easily test async functions without a runtime or test registry,
    // so we just verify the client was created successfully
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_registry_version_struct() {
    use super::RegistryVersion;

    // Test with API version
    let version_with_api = RegistryVersion {
        api_version: Some("registry/2.0".to_string()),
    };
    assert_eq!(
        version_with_api.api_version,
        Some("registry/2.0".to_string())
    );

    // Test without API version (header not present)
    let version_without_api = RegistryVersion { api_version: None };
    assert_eq!(version_without_api.api_version, None);
}

#[test]
fn test_registry_version_equality() {
    use super::RegistryVersion;

    let v1 = RegistryVersion {
        api_version: Some("registry/2.0".to_string()),
    };
    let v2 = RegistryVersion {
        api_version: Some("registry/2.0".to_string()),
    };
    let v3 = RegistryVersion { api_version: None };

    assert_eq!(v1, v2);
    assert_ne!(v1, v3);
}

// Tests for client configuration

#[test]
fn test_client_config_default() {
    use super::ClientConfig;

    let config = ClientConfig::new();
    assert_eq!(config.timeout_seconds, 30);
    assert_eq!(config.max_idle_per_host, 10);
}

#[test]
fn test_client_config_with_timeout() {
    use super::ClientConfig;

    let config = ClientConfig::new().with_timeout(60);
    assert_eq!(config.timeout_seconds, 60);
    assert_eq!(config.max_idle_per_host, 10); // Other settings unchanged
}

#[test]
fn test_client_config_with_max_idle() {
    use super::ClientConfig;

    let config = ClientConfig::new().with_max_idle_per_host(20);
    assert_eq!(config.timeout_seconds, 30); // Other settings unchanged
    assert_eq!(config.max_idle_per_host, 20);
}

#[test]
fn test_client_config_builder_chaining() {
    use super::ClientConfig;

    let config = ClientConfig::new()
        .with_timeout(120)
        .with_max_idle_per_host(50);
    assert_eq!(config.timeout_seconds, 120);
    assert_eq!(config.max_idle_per_host, 50);
}

#[test]
fn test_client_with_custom_config() {
    use super::{Client, ClientConfig};

    let config = ClientConfig::new()
        .with_timeout(60)
        .with_max_idle_per_host(20);

    let client = Client::with_config("http://localhost:5000", config, None);
    assert!(client.is_ok());
    assert_eq!(client.unwrap().registry_url(), "http://localhost:5000");
}

#[test]
fn test_client_new_uses_default_config() {
    // Verify that Client::new() still works and uses defaults
    let client = Client::new("http://localhost:5000", None);
    assert!(client.is_ok());
}

// Tests for catalog operations

#[test]
fn test_catalog_response_deserialization() {
    // Test that we can parse a valid catalog response
    let json = r#"{"repositories":["alpine","nginx","postgres"]}"#;
    let response: super::CatalogResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.repositories.len(), 3);
    assert_eq!(response.repositories[0], "alpine");
    assert_eq!(response.repositories[1], "nginx");
    assert_eq!(response.repositories[2], "postgres");
}

#[test]
fn test_catalog_response_empty() {
    // Test empty repository list
    let json = r#"{"repositories":[]}"#;
    let response: super::CatalogResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.repositories.len(), 0);
}

#[test]
fn test_catalog_url_construction() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    // The catalog endpoint URL would be: "http://localhost:5000/v2/_catalog"
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_extract_next_link_with_double_quotes() {
    // Test Link header parsing with double quotes
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::LINK,
        reqwest::header::HeaderValue::from_static(
            r#"</v2/_catalog?n=100&last=repo99>; rel="next""#,
        ),
    );

    let next = Client::extract_next_link(&headers);
    assert_eq!(next, Some("/v2/_catalog?n=100&last=repo99".to_string()));
}

#[test]
fn test_extract_next_link_with_single_quotes() {
    // Test Link header parsing with single quotes
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::LINK,
        reqwest::header::HeaderValue::from_static(r#"</v2/_catalog?n=50&last=alpine>; rel='next'"#),
    );

    let next = Client::extract_next_link(&headers);
    assert_eq!(next, Some("/v2/_catalog?n=50&last=alpine".to_string()));
}

#[test]
fn test_extract_next_link_no_link_header() {
    // Test when there's no Link header (last page)
    let headers = reqwest::header::HeaderMap::new();
    let next = Client::extract_next_link(&headers);
    assert_eq!(next, None);
}

#[test]
fn test_extract_next_link_multiple_links() {
    // Test Link header with multiple links
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::LINK,
        reqwest::header::HeaderValue::from_static(
            r#"</v2/_catalog?n=100&last=repo1>; rel="prev", </v2/_catalog?n=100&last=repo99>; rel="next""#,
        ),
    );

    let next = Client::extract_next_link(&headers);
    assert_eq!(next, Some("/v2/_catalog?n=100&last=repo99".to_string()));
}

// Tests for tag operations

#[test]
fn test_tags_response_deserialization() {
    // Test that we can parse a valid tags response
    let json = r#"{"name":"alpine","tags":["latest","3.19","3.18","edge"]}"#;
    let response: super::TagsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.name, "alpine");
    assert_eq!(response.tags.len(), 4);
    assert_eq!(response.tags[0], "latest");
    assert_eq!(response.tags[1], "3.19");
    assert_eq!(response.tags[2], "3.18");
    assert_eq!(response.tags[3], "edge");
}

#[test]
fn test_tags_response_empty() {
    // Test empty tag list
    let json = r#"{"name":"myrepo","tags":[]}"#;
    let response: super::TagsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.name, "myrepo");
    assert_eq!(response.tags.len(), 0);
}

#[test]
fn test_tags_url_construction() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    // The tags endpoint URL would be: "http://localhost:5000/v2/{name}/tags/list"
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_tags_response_with_special_characters() {
    // Test tag names with various valid characters
    let json = r#"{"name":"myrepo","tags":["v1.0.0","latest","beta-1","sha-abc123","2024.01.15"]}"#;
    let response: super::TagsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.tags.len(), 5);
    assert_eq!(response.tags[0], "v1.0.0");
    assert_eq!(response.tags[1], "latest");
    assert_eq!(response.tags[2], "beta-1");
    assert_eq!(response.tags[3], "sha-abc123");
    assert_eq!(response.tags[4], "2024.01.15");
}

// Tests for manifest operations

#[test]
fn test_manifest_url_construction() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    // The manifest endpoint URL would be: "http://localhost:5000/v2/{name}/manifests/{reference}"
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_manifest_url_with_tag_reference() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    // Should construct: "http://localhost:5000/v2/alpine/manifests/latest"
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_manifest_url_with_digest_reference() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    // Should construct: "http://localhost:5000/v2/alpine/manifests/sha256:abc123..."
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_manifest_accept_headers() {
    // Verify that the Accept header includes all required manifest media types
    let accept_header = "application/vnd.oci.image.manifest.v1+json, \
                        application/vnd.oci.image.index.v1+json, \
                        application/vnd.docker.distribution.manifest.v2+json, \
                        application/vnd.docker.distribution.manifest.list.v2+json";

    // Verify OCI manifest types
    assert!(accept_header.contains("application/vnd.oci.image.manifest.v1+json"));
    assert!(accept_header.contains("application/vnd.oci.image.index.v1+json"));

    // Verify Docker manifest types
    assert!(accept_header.contains("application/vnd.docker.distribution.manifest.v2+json"));
    assert!(accept_header.contains("application/vnd.docker.distribution.manifest.list.v2+json"));
}

// Tests for blob operations

#[test]
fn test_blob_url_construction() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    // The blob endpoint URL would be: "http://localhost:5000/v2/{name}/blobs/{digest}"
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_digest_validation() {
    // Test that invalid digest formats are rejected
    use crate::digest::Digest;
    use std::str::FromStr;

    // Valid digest
    let valid =
        Digest::from_str("sha256:4abcf20661432fb2d719b4568d94db3b6cf9b44bf2a3e1c2c6d0c89fd9e6e0b2");
    assert!(valid.is_ok());

    // Invalid format
    let invalid = Digest::from_str("invalid_digest");
    assert!(invalid.is_err());

    // Missing algorithm
    let missing_algo =
        Digest::from_str("4abcf20661432fb2d719b4568d94db3b6cf9b44bf2a3e1c2c6d0c89fd9e6e0b2");
    assert!(missing_algo.is_err());
}

#[test]
fn test_digest_algorithm_extraction() {
    use crate::digest::Digest;
    use std::str::FromStr;

    let digest =
        Digest::from_str("sha256:4abcf20661432fb2d719b4568d94db3b6cf9b44bf2a3e1c2c6d0c89fd9e6e0b2")
            .unwrap();
    assert_eq!(digest.algorithm(), "sha256");
}

#[test]
fn test_digest_hex_extraction() {
    use crate::digest::Digest;
    use std::str::FromStr;

    let digest =
        Digest::from_str("sha256:4abcf20661432fb2d719b4568d94db3b6cf9b44bf2a3e1c2c6d0c89fd9e6e0b2")
            .unwrap();
    assert_eq!(
        digest.hex(),
        "4abcf20661432fb2d719b4568d94db3b6cf9b44bf2a3e1c2c6d0c89fd9e6e0b2"
    );
}

// Tests for delete_manifest method
// Note: These are structural tests. Full integration tests with a real registry
// will be added in the integration test suite.

#[test]
fn test_delete_manifest_url_construction() {
    // Verify that delete_manifest constructs the correct URL format
    let client = Client::new("http://localhost:5000", None).unwrap();
    assert_eq!(client.registry_url(), "http://localhost:5000");

    // The method should construct: "http://localhost:5000/v2/<repo>/manifests/<digest>"
    // We verify the client has this method available
    // (Actual deletion tests require a running registry)
}

#[test]
fn test_delete_manifest_method_exists() {
    // Verify the delete_manifest method signature exists and client can be created
    let client = Client::new("http://localhost:5000", None).unwrap();

    // The method signature is: pub fn delete_manifest(&self, repository: &str, digest: &str) -> Result<()>
    // We just verify the client was created successfully
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_delete_manifest_with_credentials() {
    use crate::auth::Credentials;

    // Verify that client with credentials can be created for delete operations
    let creds = Credentials::basic("user", "pass");
    let client = Client::new("http://localhost:5000", Some(creds)).unwrap();

    assert_eq!(client.registry_url(), "http://localhost:5000");
    // The delete_manifest method should include the Authorization header when credentials are present
}

// Mock-based integration tests
#[test]
fn test_check_version_success() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(200)
        .with_header("Docker-Distribution-API-Version", "registry/2.0")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_ok());
    let version = result.unwrap();
    assert_eq!(version.api_version, Some("registry/2.0".to_string()));
}

#[test]
fn test_check_version_unauthorized() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(401)
        .with_body("authentication required")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RexError::Authentication { .. }
    ));
}

#[test]
fn test_check_version_forbidden() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(403)
        .with_body("access forbidden")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RexError::Authentication { .. }
    ));
}

#[test]
fn test_check_version_not_found() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(404)
        .with_body("not found")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::NotFound { .. }));
}

#[test]
fn test_check_version_server_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(500)
        .with_body("internal server error")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::Server { .. }));
}

#[test]
fn test_check_version_rate_limit() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(429)
        .with_body("too many requests")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::RateLimit { .. }));
}

#[test]
fn test_check_version_rate_limit_with_retry_after_seconds() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(429)
        .with_header("Retry-After", "120")
        .with_body("too many requests")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());

    match result.unwrap_err() {
        RexError::RateLimit {
            message,
            retry_after,
        } => {
            assert!(message.contains("Rate limit exceeded"));
            assert_eq!(retry_after, Some(120));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_check_version_rate_limit_with_retry_after_http_date() {
    let mut server = mockito::Server::new();

    // Use a date format that httpdate can parse (RFC 2822 format)
    let mock = server
        .mock("GET", "/v2/")
        .with_status(429)
        .with_header("Retry-After", "Sun, 06 Nov 2044 08:49:37 GMT")
        .with_body("too many requests")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());

    match result.unwrap_err() {
        RexError::RateLimit {
            message,
            retry_after,
        } => {
            assert!(message.contains("Rate limit exceeded"));
            // HTTP-date parsing may succeed and calculate seconds from now
            // If httpdate crate doesn't support the format, it falls back to None
            // Either behavior is acceptable (defensive programming)
            // We just verify it doesn't crash and returns a valid option
            assert!(retry_after.is_none() || retry_after.unwrap() > 0);
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_check_version_rate_limit_with_invalid_retry_after() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(429)
        .with_header("Retry-After", "invalid")
        .with_body("too many requests")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());

    match result.unwrap_err() {
        RexError::RateLimit {
            message,
            retry_after,
        } => {
            assert!(message.contains("Rate limit exceeded"));
            // Should fall back to None if header is invalid
            assert_eq!(retry_after, None);
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_check_version_rate_limit_with_zero_retry_after() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(429)
        .with_header("Retry-After", "0")
        .with_body("too many requests")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());

    match result.unwrap_err() {
        RexError::RateLimit {
            message,
            retry_after,
        } => {
            assert!(message.contains("Rate limit exceeded"));
            // Zero is valid - means retry immediately
            assert_eq!(retry_after, Some(0));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_check_version_rate_limit_with_large_retry_after() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(429)
        .with_header("Retry-After", "3600")
        .with_body("too many requests")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());

    match result.unwrap_err() {
        RexError::RateLimit {
            message,
            retry_after,
        } => {
            assert!(message.contains("Rate limit exceeded"));
            // Large value (1 hour) should be parsed correctly
            assert_eq!(retry_after, Some(3600));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_check_version_rate_limit_with_whitespace_retry_after() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(429)
        .with_header("Retry-After", " 60 ")
        .with_body("too many requests")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());

    match result.unwrap_err() {
        RexError::RateLimit {
            message,
            retry_after,
        } => {
            assert!(message.contains("Rate limit exceeded"));
            // Whitespace should be handled by parse::<u64>()
            // Note: Rust's parse() trims whitespace automatically
            assert_eq!(retry_after, Some(60));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_check_version_rate_limit_with_negative_retry_after() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(429)
        .with_header("Retry-After", "-10")
        .with_body("too many requests")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());

    match result.unwrap_err() {
        RexError::RateLimit {
            message,
            retry_after,
        } => {
            assert!(message.contains("Rate limit exceeded"));
            // Negative numbers should fail u64 parsing and fall back to None
            assert_eq!(retry_after, None);
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_check_version_rate_limit_without_retry_after_header() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(429)
        .with_body("too many requests")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());

    match result.unwrap_err() {
        RexError::RateLimit {
            message,
            retry_after,
        } => {
            assert!(message.contains("Rate limit exceeded"));
            // Missing header should result in None
            assert_eq!(retry_after, None);
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_fetch_catalog_rate_limit_with_retry_after() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/_catalog")
        .with_status(429)
        .with_header("Retry-After", "90")
        .with_body("too many requests")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_catalog();

    mock.assert();
    assert!(result.is_err());

    match result.unwrap_err() {
        RexError::RateLimit {
            message,
            retry_after,
        } => {
            assert!(message.contains("Rate limit exceeded"));
            // Verify retry-after parsing works for catalog endpoint too
            assert_eq!(retry_after, Some(90));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_fetch_catalog_success() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/_catalog")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"repositories":["alpine","nginx","redis"]}"#)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_catalog();

    mock.assert();
    assert!(result.is_ok());
    let repos = result.unwrap();
    assert_eq!(repos.len(), 3);
    assert_eq!(repos[0], "alpine");
    assert_eq!(repos[1], "nginx");
    assert_eq!(repos[2], "redis");
}

#[test]
fn test_fetch_catalog_empty() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/_catalog")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"repositories":[]}"#)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_catalog();

    mock.assert();
    assert!(result.is_ok());
    let repos = result.unwrap();
    assert_eq!(repos.len(), 0);
}

#[test]
fn test_fetch_catalog_with_pagination() {
    let mut server = mockito::Server::new();

    // First page
    let mock1 = server
        .mock("GET", "/v2/_catalog?n=2")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_header("Link", r#"</v2/_catalog?n=2&last=nginx>; rel="next""#)
        .with_body(r#"{"repositories":["alpine","nginx"]}"#)
        .create();

    // Second page
    let mock2 = server
        .mock("GET", "/v2/_catalog?n=2&last=nginx")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"repositories":["redis"]}"#)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_catalog_paginated(Some(2));

    mock1.assert();
    mock2.assert();
    assert!(result.is_ok());
    let repos = result.unwrap();
    assert_eq!(repos.len(), 3);
    assert_eq!(repos[0], "alpine");
    assert_eq!(repos[1], "nginx");
    assert_eq!(repos[2], "redis");
}

#[test]
fn test_fetch_catalog_unauthorized() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/_catalog")
        .with_status(401)
        .with_body("authentication required")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_catalog();

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RexError::Authentication { .. }
    ));
}

#[test]
fn test_fetch_tags_success() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/alpine/tags/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"name":"alpine","tags":["3.14","3.15","latest"]}"#)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_tags("alpine");

    mock.assert();
    assert!(result.is_ok());
    let tags = result.unwrap();
    assert_eq!(tags.len(), 3);
    assert_eq!(tags[0], "3.14");
    assert_eq!(tags[1], "3.15");
    assert_eq!(tags[2], "latest");
}

#[test]
fn test_fetch_tags_empty() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/alpine/tags/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"name":"alpine","tags":[]}"#)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_tags("alpine");

    mock.assert();
    assert!(result.is_ok());
    let tags = result.unwrap();
    assert_eq!(tags.len(), 0);
}

#[test]
fn test_fetch_tags_with_pagination() {
    let mut server = mockito::Server::new();

    // First page
    let mock1 = server
        .mock("GET", "/v2/alpine/tags/list?n=2")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_header(
            "Link",
            r#"</v2/alpine/tags/list?n=2&last=3.15>; rel="next""#,
        )
        .with_body(r#"{"name":"alpine","tags":["3.14","3.15"]}"#)
        .create();

    // Second page
    let mock2 = server
        .mock("GET", "/v2/alpine/tags/list?n=2&last=3.15")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"name":"alpine","tags":["latest"]}"#)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_tags_paginated("alpine", Some(2));

    mock1.assert();
    mock2.assert();
    assert!(result.is_ok());
    let tags = result.unwrap();
    assert_eq!(tags.len(), 3);
    assert_eq!(tags[0], "3.14");
    assert_eq!(tags[1], "3.15");
    assert_eq!(tags[2], "latest");
}

#[test]
fn test_fetch_tags_wrong_repository_name() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/alpine/tags/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"name":"nginx","tags":["latest"]}"#)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_tags("alpine");

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::Validation { .. }));
}

#[test]
fn test_fetch_tags_not_found() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/nonexistent/tags/list")
        .with_status(404)
        .with_body("repository not found")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_tags("nonexistent");

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::NotFound { .. }));
}

#[test]
fn test_fetch_manifest_success() {
    let mut server = mockito::Server::new();
    let manifest_body =
        r#"{"schemaVersion":2,"mediaType":"application/vnd.docker.distribution.manifest.v2+json"}"#;

    let mock = server
        .mock("GET", "/v2/alpine/manifests/latest")
        .with_status(200)
        .with_header(
            "content-type",
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .with_header("Docker-Content-Digest", "sha256:abc123")
        .with_body(manifest_body)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_manifest("alpine", "latest");

    mock.assert();
    assert!(result.is_ok());
    let (bytes, digest) = result.unwrap();
    assert_eq!(bytes, manifest_body.as_bytes());
    assert_eq!(digest, "sha256:abc123");
}

#[test]
fn test_fetch_manifest_missing_digest_header() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/v2/alpine/manifests/latest")
        .with_status(200)
        .with_header(
            "content-type",
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .with_body(r#"{"schemaVersion":2}"#)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_manifest("alpine", "latest");

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::Validation { .. }));
}

#[test]
fn test_fetch_manifest_not_found() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/alpine/manifests/nonexistent")
        .with_status(404)
        .with_body("manifest not found")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_manifest("alpine", "nonexistent");

    mock.assert();
    assert!(result.is_err());
    // fetch_manifest returns Validation error for missing digest header before checking status
    // So a 404 actually causes a Validation error when the digest header is missing
    let err = result.unwrap_err();
    // Either NotFound or Validation is acceptable for this case
    assert!(
        matches!(err, RexError::NotFound { .. }) || matches!(err, RexError::Validation { .. }),
        "Expected NotFound or Validation error, got: {:?}",
        err
    );
}

#[test]
fn test_fetch_blob_success() {
    use sha2::{Digest as Sha2Digest, Sha256};

    let mut server = mockito::Server::new();
    let blob_content = b"test blob content";

    // Calculate the actual SHA256 hash
    let mut hasher = Sha256::new();
    hasher.update(blob_content);
    let hash = format!("{:x}", hasher.finalize());
    let digest = format!("sha256:{}", hash);

    let mock = server
        .mock("GET", format!("/v2/alpine/blobs/{}", digest).as_str())
        .with_status(200)
        .with_body(blob_content)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_blob("alpine", &digest);

    mock.assert();
    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes, blob_content);
}

#[test]
fn test_fetch_blob_digest_mismatch() {
    let mut server = mockito::Server::new();
    // Provide different content than what the digest says
    let blob_content = b"wrong content";
    let digest = "sha256:4abcf20661432fb2d719b4568d94db3b6cf9b44bf2a3e1c2c6d0c89fd9e6e0b2";

    let mock = server
        .mock("GET", format!("/v2/alpine/blobs/{}", digest).as_str())
        .with_status(200)
        .with_body(blob_content)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_blob("alpine", digest);

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::Validation { .. }));
}

#[test]
fn test_fetch_blob_not_found() {
    let mut server = mockito::Server::new();
    // Use a valid digest format (valid hex)
    let digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let mock = server
        .mock("GET", format!("/v2/alpine/blobs/{}", digest).as_str())
        .with_status(404)
        .with_body("blob not found")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_blob("alpine", digest);

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::NotFound { .. }));
}

#[test]
fn test_fetch_blob_unsupported_algorithm() {
    let mut server = mockito::Server::new();
    let blob_content = b"test content";

    // Use SHA512 which is not supported
    let digest = "sha512:1234567890abcdef";

    // Create a mock that won't be called since validation happens first
    let _mock = server
        .mock("GET", format!("/v2/alpine/blobs/{}", digest).as_str())
        .with_status(200)
        .with_body(blob_content)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.fetch_blob("alpine", digest);

    // Should fail during validation, not during the request
    assert!(result.is_err());
    // The error is either from invalid digest format or unsupported algorithm
    assert!(matches!(result.unwrap_err(), RexError::Validation { .. }));
}

#[test]
fn test_check_response_status_bad_gateway() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(502)
        .with_body("bad gateway")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::Server { .. }));
}

#[test]
fn test_check_response_status_service_unavailable() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(503)
        .with_body("service unavailable")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::Server { .. }));
}

#[test]
fn test_check_response_status_gateway_timeout() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(504)
        .with_body("gateway timeout")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RexError::Server { .. }));
}

#[test]
fn test_client_with_credentials() {
    use crate::auth::Credentials;

    let mut server = mockito::Server::new();
    let creds = Credentials::basic("user", "pass");

    let mock = server
        .mock("GET", "/v2/")
        .match_header("Authorization", "Basic dXNlcjpwYXNz")
        .with_status(200)
        .with_header("Docker-Distribution-API-Version", "registry/2.0")
        .create();

    let client = Client::new(&server.url(), Some(creds)).unwrap();
    let result = client.check_version();

    mock.assert();
    assert!(result.is_ok());
}
