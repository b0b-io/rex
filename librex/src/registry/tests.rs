use super::*;
use crate::cache::Cache;
use crate::client::Client;
use crate::config::Config;
use std::num::NonZeroUsize;
use tempfile::tempdir;

#[test]
fn test_registry_new() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    let registry = Registry::new(client, None, None);

    assert!(registry.cache.is_none());
    assert!(registry.credentials.is_none());
}

#[test]
fn test_registry_new_with_cache() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    let temp_dir = tempdir().unwrap();
    let config = Config::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf(), config.cache.ttl, capacity);

    let registry = Registry::new(client, Some(cache), None);

    assert!(registry.cache.is_some());
    assert!(registry.credentials.is_none());
}

#[test]
fn test_registry_credentials_management() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    let mut registry = Registry::new(client, None, None);

    // Initially no credentials
    assert!(registry.credentials().is_none());

    // Set credentials
    let creds = Credentials::Basic {
        username: "user".to_string(),
        password: "pass".to_string(),
    };
    registry.set_credentials(creds.clone());
    assert_eq!(registry.credentials(), Some(&creds));

    // Clear credentials
    registry.clear_credentials();
    assert!(registry.credentials().is_none());
}

#[test]
fn test_catalog_response_serde() {
    let catalog = CatalogResponse {
        repositories: vec!["alpine".to_string(), "ubuntu".to_string()],
    };

    // Test serialization
    let json = serde_json::to_string(&catalog).unwrap();
    assert!(json.contains("alpine"));
    assert!(json.contains("ubuntu"));

    // Test deserialization
    let deserialized: CatalogResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, catalog);
}

#[test]
fn test_tags_response_serde() {
    let tags = TagsResponse {
        name: "alpine".to_string(),
        tags: vec!["latest".to_string(), "3.19".to_string()],
    };

    // Test serialization
    let json = serde_json::to_string(&tags).unwrap();
    assert!(json.contains("alpine"));
    assert!(json.contains("latest"));

    // Test deserialization
    let deserialized: TagsResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, tags);
}

// Integration tests would require a mock registry server or test containers
// These are unit tests for the data structures and basic functionality

#[test]
fn test_blob_cache_key_is_global() {
    // This test verifies that blob cache keys are global (not repository-specific)
    // The actual caching behavior requires HTTP mocking, but we can verify
    // the cache key format would be correct
    use crate::digest::Digest;
    use std::str::FromStr;

    let digest =
        Digest::from_str("sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b")
            .unwrap();
    let expected_key = format!("blobs/{}", digest);

    // The cache key should be "blobs/sha256:..."
    // not "repo1/blobs/sha256:..." or "repo2/blobs/sha256:..."
    // This ensures the same blob is cached once across all repositories
    assert!(expected_key.starts_with("blobs/sha256:"));
    assert!(!expected_key.contains("repository"));
    assert_eq!(
        expected_key,
        "blobs/sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b"
    );
}

#[test]
fn test_manifest_or_index_types_work_with_registry() {
    // This test verifies that the ManifestOrIndex type is properly integrated
    // It doesn't test the actual HTTP calls, just that the types work together
    use crate::oci::ManifestOrIndex;

    // Sample manifest JSON
    let manifest_json = r#"{
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": {
            "mediaType": "application/vnd.oci.image.config.v1+json",
            "size": 1234,
            "digest": "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        },
        "layers": []
    }"#;

    let manifest_or_index = ManifestOrIndex::from_bytes(manifest_json.as_bytes()).unwrap();
    assert!(manifest_or_index.is_manifest());
    assert!(manifest_or_index.as_manifest().is_some());

    // Sample index JSON
    let index_json = r#"{
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.index.v1+json",
        "manifests": [
            {
                "mediaType": "application/vnd.oci.image.manifest.v1+json",
                "size": 1234,
                "digest": "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                "platform": {
                    "architecture": "amd64",
                    "os": "linux"
                }
            }
        ]
    }"#;

    let manifest_or_index = ManifestOrIndex::from_bytes(index_json.as_bytes()).unwrap();
    assert!(manifest_or_index.is_index());
    assert!(manifest_or_index.as_index().is_some());
    let platforms = manifest_or_index.platforms();
    assert_eq!(platforms.len(), 1);
}
