use super::*;
use crate::cache::Cache;
use crate::client::Client;
use crate::config::Config;
use std::num::NonZeroUsize;
use tempfile::tempdir;

#[test]
fn test_registry_new() {
    let client = Client::new("http://localhost:5000").unwrap();
    let registry = Registry::new(client, None, None);

    assert!(registry.cache.is_none());
    assert!(registry.credentials.is_none());
}

#[test]
fn test_registry_new_with_cache() {
    let client = Client::new("http://localhost:5000").unwrap();
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
    let client = Client::new("http://localhost:5000").unwrap();
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
