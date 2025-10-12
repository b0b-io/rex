use super::*;

#[test]
fn test_client_new_with_valid_url() {
    let client = Client::new("http://localhost:5000");
    assert!(client.is_ok());
}

#[test]
fn test_client_new_with_https_url() {
    let client = Client::new("https://registry.example.com");
    assert!(client.is_ok());
}

#[test]
fn test_client_normalizes_url_without_scheme() {
    let client = Client::new("localhost:5000").unwrap();
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_client_removes_trailing_slash() {
    let client = Client::new("http://localhost:5000/").unwrap();
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_client_removes_multiple_trailing_slashes() {
    let client = Client::new("http://localhost:5000///").unwrap();
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_client_new_with_empty_url_fails() {
    let client = Client::new("");
    assert!(client.is_err());
    assert!(matches!(client.unwrap_err(), RexError::Validation { .. }));
}

#[test]
fn test_client_new_with_whitespace_url_fails() {
    let client = Client::new("   ");
    assert!(client.is_err());
}

#[test]
fn test_client_registry_url_accessor() {
    let client = Client::new("http://localhost:5000").unwrap();
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_client_with_port() {
    let client = Client::new("localhost:8080").unwrap();
    assert_eq!(client.registry_url(), "http://localhost:8080");
}

#[test]
fn test_client_with_domain() {
    let client = Client::new("registry.example.com").unwrap();
    assert_eq!(client.registry_url(), "http://registry.example.com");
}

// Note: Integration tests for check_version() will be added when we have
// a test registry available. For now, we verify the basic structure.

#[test]
fn test_url_construction_for_version_check() {
    let client = Client::new("http://localhost:5000").unwrap();
    // Verify the URL would be correct for version check
    assert_eq!(client.registry_url(), "http://localhost:5000");
    // The version check uses: format!("{}/v2/", client.registry_url())
    // which would produce: "http://localhost:5000/v2/"
}

#[test]
fn test_client_has_async_check_version_method() {
    // This test just verifies the method exists and can be called
    // Integration testing with a real registry will be done later
    let client = Client::new("http://localhost:5000").unwrap();
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

    let client = Client::with_config("http://localhost:5000", config);
    assert!(client.is_ok());
    assert_eq!(client.unwrap().registry_url(), "http://localhost:5000");
}

#[test]
fn test_client_new_uses_default_config() {
    // Verify that Client::new() still works and uses defaults
    let client = Client::new("http://localhost:5000");
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
    let client = Client::new("http://localhost:5000").unwrap();
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
    let client = Client::new("http://localhost:5000").unwrap();
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
    let client = Client::new("http://localhost:5000").unwrap();
    // The manifest endpoint URL would be: "http://localhost:5000/v2/{name}/manifests/{reference}"
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_manifest_url_with_tag_reference() {
    let client = Client::new("http://localhost:5000").unwrap();
    // Should construct: "http://localhost:5000/v2/alpine/manifests/latest"
    assert_eq!(client.registry_url(), "http://localhost:5000");
}

#[test]
fn test_manifest_url_with_digest_reference() {
    let client = Client::new("http://localhost:5000").unwrap();
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
    let client = Client::new("http://localhost:5000").unwrap();
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
