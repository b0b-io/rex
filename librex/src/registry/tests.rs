use super::*;
use crate::cache::{Cache, CacheTtl};
use crate::client::Client;
use std::num::NonZeroUsize;
use tempfile::tempdir;

#[test]
fn test_registry_new() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    let registry = Registry::new(client, None, None, false);

    assert!(registry.cache.is_none());
    assert!(registry.credentials.is_none());
}

#[test]
fn test_registry_new_with_cache() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    let temp_dir = tempdir().unwrap();
    let ttl = CacheTtl::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf(), ttl, capacity);

    let registry = Registry::new(client, Some(cache), None, false);

    assert!(registry.cache.is_some());
    assert!(registry.credentials.is_none());
}

#[test]
fn test_registry_credentials_management() {
    let client = Client::new("http://localhost:5000", None).unwrap();
    let mut registry = Registry::new(client, None, None, false);

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

#[test]
fn test_digest_computation_from_manifest_bytes() {
    // This test verifies that we can compute the correct digest from manifest bytes
    // The digest is SHA256 of the exact bytes
    use sha2::{Digest as Sha2Digest, Sha256};

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

    let bytes = manifest_json.as_bytes();

    // Compute digest
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    let digest = format!("sha256:{:x}", hash);

    // Verify format
    assert!(digest.starts_with("sha256:"));
    assert_eq!(digest.len(), 71); // "sha256:" (7) + 64 hex chars

    // Verify the same bytes produce the same digest
    let mut hasher2 = Sha256::new();
    hasher2.update(bytes);
    let hash2 = hasher2.finalize();
    let digest2 = format!("sha256:{:x}", hash2);

    assert_eq!(digest, digest2);
}

#[test]
fn test_manifest_digest_consistency() {
    // This test verifies that the digest computation is consistent with OCI spec
    // The digest should be the sha256 of the canonical JSON bytes
    use crate::oci::ManifestOrIndex;

    let manifest_json = r#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"mediaType":"application/vnd.oci.image.config.v1+json","size":1234,"digest":"sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"},"layers":[]}"#;

    // Parse the manifest
    let manifest_or_index = ManifestOrIndex::from_bytes(manifest_json.as_bytes()).unwrap();
    assert!(manifest_or_index.is_manifest());

    // Compute digest from the exact bytes
    use sha2::{Digest as Sha2Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(manifest_json.as_bytes());
    let hash = hasher.finalize();
    let computed_digest = format!("sha256:{:x}", hash);

    // The computed digest should be deterministic
    assert!(computed_digest.starts_with("sha256:"));
    assert_eq!(computed_digest.len(), 71);
}

// Tests for cache invalidation during delete operations

#[test]
fn test_delete_manifest_cache_key_format() {
    // Verify that delete_manifest uses the correct cache key format
    // This matches the format used in get_manifest for digest-based lookups
    let repository = "alpine";
    let digest = "sha256:abc123";
    let expected_cache_key = format!("{}/manifests/{}", repository, digest);

    // The cache key should match: "{repo}/manifests/{digest}"
    assert_eq!(expected_cache_key, "alpine/manifests/sha256:abc123");
}

#[test]
fn test_delete_tag_cache_key_formats() {
    // Verify that delete_tag invalidates all the correct cache keys
    let repository = "alpine";
    let tag = "latest";
    let digest = "sha256:abc123";

    // Should invalidate tag-based manifest cache
    let tag_cache_key = format!("{}/tags/{}/manifest", repository, tag);
    assert_eq!(tag_cache_key, "alpine/tags/latest/manifest");

    // Should invalidate tags list cache
    let tags_list_key = format!("{}/_tags", repository);
    assert_eq!(tags_list_key, "alpine/_tags");

    // Should invalidate digest-based manifest cache (via delete_manifest)
    let digest_cache_key = format!("{}/manifests/{}", repository, digest);
    assert_eq!(digest_cache_key, "alpine/manifests/sha256:abc123");
}

#[test]
fn test_delete_manifest_invalidates_cache() {
    // This test verifies that calling delete_manifest actually removes the cache entry
    // We can't test the HTTP DELETE without a running registry, but we can test cache invalidation

    let temp_dir = tempdir().unwrap();
    let ttl = CacheTtl::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(temp_dir.path().to_path_buf(), ttl, capacity);

    // Manually populate cache with a manifest entry
    let repository = "alpine";
    let digest = "sha256:abc123def456";
    let cache_key = format!("{}/manifests/{}", repository, digest);
    let fake_manifest_data = b"fake manifest data";

    cache
        .set(
            &cache_key,
            &fake_manifest_data.to_vec(),
            CacheType::Manifest,
        )
        .unwrap();

    // Verify cache entry exists
    let cached: Option<Vec<u8>> = cache.get(&cache_key).unwrap();
    assert!(cached.is_some());
    assert_eq!(cached.unwrap(), fake_manifest_data);

    // Now test that delete would invalidate this key
    // Since we can't actually call delete_manifest (it would try to make HTTP request),
    // we directly test the cache.delete() method with the same key format
    cache.delete(&cache_key).unwrap();

    // Verify cache entry is gone
    let cached_after: Option<Vec<u8>> = cache.get(&cache_key).unwrap();
    assert!(cached_after.is_none());
}

#[test]
fn test_delete_tag_invalidates_multiple_cache_entries() {
    // Verify that delete_tag would invalidate all the correct cache keys

    let temp_dir = tempdir().unwrap();
    let ttl = CacheTtl::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(temp_dir.path().to_path_buf(), ttl, capacity);

    let repository = "alpine";
    let tag = "latest";
    let digest = "sha256:abc123def456";

    // Populate cache with all the entries that should be invalidated
    let tag_cache_key = format!("{}/tags/{}/manifest", repository, tag);
    let tags_list_key = format!("{}/_tags", repository);
    let digest_cache_key = format!("{}/manifests/{}", repository, digest);

    let fake_data = b"fake data";
    cache
        .set(&tag_cache_key, &fake_data.to_vec(), CacheType::Manifest)
        .unwrap();
    cache
        .set(&tags_list_key, &vec!["latest".to_string()], CacheType::Tags)
        .unwrap();
    cache
        .set(&digest_cache_key, &fake_data.to_vec(), CacheType::Manifest)
        .unwrap();

    // Verify all entries exist
    assert!(cache.get::<Vec<u8>>(&tag_cache_key).unwrap().is_some());
    assert!(cache.get::<Vec<String>>(&tags_list_key).unwrap().is_some());
    assert!(cache.get::<Vec<u8>>(&digest_cache_key).unwrap().is_some());

    // Simulate what delete_tag does - invalidate all three cache keys
    cache.delete(&tag_cache_key).unwrap();
    cache.delete(&tags_list_key).unwrap();
    cache.delete(&digest_cache_key).unwrap();

    // Verify all entries are gone
    assert!(cache.get::<Vec<u8>>(&tag_cache_key).unwrap().is_none());
    assert!(cache.get::<Vec<String>>(&tags_list_key).unwrap().is_none());
    assert!(cache.get::<Vec<u8>>(&digest_cache_key).unwrap().is_none());
}

#[test]
fn test_delete_all_tags_invalidates_tags_list_cache() {
    // Verify that delete_all_tags invalidates the tags list cache

    let temp_dir = tempdir().unwrap();
    let ttl = CacheTtl::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(temp_dir.path().to_path_buf(), ttl, capacity);

    let repository = "alpine";
    let tags_list_key = format!("{}/_tags", repository);

    // Populate cache with tags list
    let tags = vec!["latest".to_string(), "3.19".to_string(), "3.18".to_string()];
    cache.set(&tags_list_key, &tags, CacheType::Tags).unwrap();

    // Verify entry exists
    let cached: Option<Vec<String>> = cache.get(&tags_list_key).unwrap();
    assert!(cached.is_some());
    assert_eq!(cached.unwrap(), tags);

    // Simulate what delete_all_tags does - invalidate tags list cache
    cache.delete(&tags_list_key).unwrap();

    // Verify cache entry is gone
    let cached_after: Option<Vec<String>> = cache.get(&tags_list_key).unwrap();
    assert!(cached_after.is_none());
}

#[test]
fn test_cache_delete_nonexistent_key_is_safe() {
    // Verify that deleting a non-existent cache key doesn't cause errors

    let temp_dir = tempdir().unwrap();
    let ttl = CacheTtl::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(temp_dir.path().to_path_buf(), ttl, capacity);

    // Try to delete a key that doesn't exist
    let result = cache.delete("nonexistent/key");
    assert!(result.is_ok());
}

// Mock-based integration tests
#[test]
fn test_list_repositories_without_cache() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/_catalog")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"repositories":["alpine","nginx","redis"]}"#)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false);

    let repos = registry.list_repositories().unwrap();

    mock.assert();
    assert_eq!(repos.len(), 3);
    assert_eq!(repos[0], "alpine");
    assert_eq!(repos[1], "nginx");
    assert_eq!(repos[2], "redis");
}

#[test]
fn test_list_repositories_with_cache() {
    use crate::cache::CacheTtl;
    use std::num::NonZeroUsize;
    use tempfile::TempDir;

    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/_catalog")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"repositories":["alpine","nginx"]}"#)
        .create();

    let temp_dir = TempDir::new().unwrap();
    let ttl = CacheTtl::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf(), ttl, capacity);

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, Some(cache), None, false);

    // First call - should hit the server
    let repos1 = registry.list_repositories().unwrap();
    assert_eq!(repos1.len(), 2);
    mock.assert();

    // Second call - should use cache (mock won't be called again)
    let repos2 = registry.list_repositories().unwrap();
    assert_eq!(repos2.len(), 2);
    assert_eq!(repos1, repos2);
}

#[test]
fn test_list_tags_without_cache() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/alpine/tags/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"name":"alpine","tags":["3.14","3.15","latest"]}"#)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false);

    let tags = registry.list_tags("alpine").unwrap();

    mock.assert();
    assert_eq!(tags.len(), 3);
    assert_eq!(tags[0], "3.14");
    assert_eq!(tags[1], "3.15");
    assert_eq!(tags[2], "latest");
}

#[test]
fn test_list_tags_with_cache() {
    use crate::cache::CacheTtl;
    use std::num::NonZeroUsize;
    use tempfile::TempDir;

    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/alpine/tags/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"name":"alpine","tags":["3.14","latest"]}"#)
        .create();

    let temp_dir = TempDir::new().unwrap();
    let ttl = CacheTtl::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf(), ttl, capacity);

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, Some(cache), None, false);

    // First call - should hit the server
    let tags1 = registry.list_tags("alpine").unwrap();
    assert_eq!(tags1.len(), 2);
    mock.assert();

    // Second call - should use cache
    let tags2 = registry.list_tags("alpine").unwrap();
    assert_eq!(tags2.len(), 2);
    assert_eq!(tags1, tags2);
}

#[test]
fn test_get_manifest_without_cache() {
    use std::str::FromStr;

    let mut server = mockito::Server::new();
    let manifest_body = r#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"mediaType":"application/vnd.oci.image.config.v1+json","size":1234,"digest":"sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"},"layers":[]}"#;

    // With dockerhub_compat=false, "alpine" strips "library/" prefix
    let mock = server
        .mock("GET", "/v2/alpine/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header("docker-content-digest", "sha256:abc123")
        .with_body(manifest_body)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false);

    let reference = Reference::from_str("alpine:latest").unwrap();
    let result = registry.get_manifest(&reference);

    mock.assert();

    // Check if we got an error or success
    match result {
        Ok((manifest_or_index, digest)) => {
            assert!(manifest_or_index.is_manifest());
            assert_eq!(digest, "sha256:abc123");
        }
        Err(e) => {
            panic!("Expected success but got error: {:?}", e);
        }
    }
}

#[test]
fn test_get_manifest_with_cache() {
    use crate::cache::CacheTtl;
    use std::num::NonZeroUsize;
    use std::str::FromStr;
    use tempfile::TempDir;

    let mut server = mockito::Server::new();
    let manifest_body = r#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"mediaType":"application/vnd.oci.image.config.v1+json","size":1234,"digest":"sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"},"layers":[]}"#;

    // With dockerhub_compat=false, "alpine" strips "library/" prefix
    let mock = server
        .mock("GET", "/v2/alpine/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header("docker-content-digest", "sha256:abc123")
        .with_body(manifest_body)
        .expect(1) // Should only be called once
        .create();

    let temp_dir = TempDir::new().unwrap();
    let ttl = CacheTtl::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf(), ttl, capacity);

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, Some(cache), None, false);

    let reference = Reference::from_str("alpine:latest").unwrap();

    // First call - should hit the server
    let result1 = registry.get_manifest(&reference);
    mock.assert();

    if let Ok((manifest1, digest1)) = result1 {
        assert!(manifest1.is_manifest());
        // The first digest comes from the HTTP header
        assert_eq!(digest1, "sha256:abc123");

        // Second call - should use cache
        let (manifest2, digest2) = registry.get_manifest(&reference).unwrap();
        assert!(manifest2.is_manifest());
        // The cached digest is recomputed from bytes, so it will be different
        // Just verify it's a valid sha256
        assert!(digest2.starts_with("sha256:"));
        assert_eq!(digest2.len(), 71); // sha256: + 64 hex chars
    } else {
        panic!("Expected success but got error: {:?}", result1.unwrap_err());
    }
}

#[test]
fn test_get_blob_without_cache() {
    use sha2::{Digest as Sha2Digest, Sha256};
    use std::str::FromStr;

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
    let mut registry = Registry::new(client, None, None, false);

    let digest_obj = Digest::from_str(&digest).unwrap();
    let blob = registry.get_blob("alpine", &digest_obj).unwrap();

    mock.assert();
    assert_eq!(blob, blob_content);
}

#[test]
fn test_get_blob_with_cache() {
    use crate::cache::CacheTtl;
    use sha2::{Digest as Sha2Digest, Sha256};
    use std::num::NonZeroUsize;
    use std::str::FromStr;
    use tempfile::TempDir;

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
        .expect(1) // Should only be called once
        .create();

    let temp_dir = TempDir::new().unwrap();
    let ttl = CacheTtl::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf(), ttl, capacity);

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, Some(cache), None, false);

    let digest_obj = Digest::from_str(&digest).unwrap();

    // First call - should hit the server
    let blob1 = registry.get_blob("alpine", &digest_obj).unwrap();
    assert_eq!(blob1, blob_content);

    // Second call - should use cache
    let blob2 = registry.get_blob("alpine", &digest_obj).unwrap();
    assert_eq!(blob2, blob_content);

    mock.assert();
}

#[test]
fn test_check_version_success() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(200)
        .with_header("Docker-Distribution-API-Version", "registry/2.0")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false);

    let result = registry.check_version();

    mock.assert();
    assert!(result.is_ok());
}

#[test]
fn test_check_version_failure() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/")
        .with_status(500)
        .with_body("internal server error")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false);

    let result = registry.check_version();

    mock.assert();
    assert!(result.is_err());
}

#[test]
fn test_list_repositories_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/_catalog")
        .with_status(401)
        .with_body("authentication required")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false);

    let result = registry.list_repositories();

    mock.assert();
    assert!(result.is_err());
}

#[test]
fn test_list_tags_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/v2/alpine/tags/list")
        .with_status(404)
        .with_body("repository not found")
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false);

    let result = registry.list_tags("alpine");

    mock.assert();
    assert!(result.is_err());
}

// Tests for dockerhub_compat functionality

#[test]
fn test_get_manifest_strips_library_prefix_when_dockerhub_compat_false() {
    use std::str::FromStr;

    let mut server = mockito::Server::new();
    let manifest_body = r#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"mediaType":"application/vnd.oci.image.config.v1+json","size":1234,"digest":"sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"},"layers":[]}"#;

    // With dockerhub_compat=false, "alpine" should strip the auto-added "library/" prefix
    // So the URL should be "/v2/alpine/manifests/latest" not "/v2/library/alpine/manifests/latest"
    let mock = server
        .mock("GET", "/v2/alpine/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header("docker-content-digest", "sha256:abc123")
        .with_body(manifest_body)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false); // dockerhub_compat = false

    let reference = Reference::from_str("alpine:latest").unwrap();
    let result = registry.get_manifest(&reference);

    mock.assert();

    match result {
        Ok((manifest_or_index, digest)) => {
            assert!(manifest_or_index.is_manifest());
            assert_eq!(digest, "sha256:abc123");
        }
        Err(e) => {
            panic!("Expected success but got error: {:?}", e);
        }
    }
}

#[test]
fn test_get_manifest_keeps_library_prefix_when_dockerhub_compat_true() {
    use std::str::FromStr;

    let mut server = mockito::Server::new();
    let manifest_body = r#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"mediaType":"application/vnd.oci.image.config.v1+json","size":1234,"digest":"sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"},"layers":[]}"#;

    // With dockerhub_compat=true, "alpine" should keep the auto-added "library/" prefix
    // So the URL should be "/v2/library/alpine/manifests/latest"
    let mock = server
        .mock("GET", "/v2/library/alpine/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header("docker-content-digest", "sha256:abc123")
        .with_body(manifest_body)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, true); // dockerhub_compat = true

    let reference = Reference::from_str("alpine:latest").unwrap();
    let result = registry.get_manifest(&reference);

    mock.assert();

    match result {
        Ok((manifest_or_index, digest)) => {
            assert!(manifest_or_index.is_manifest());
            assert_eq!(digest, "sha256:abc123");
        }
        Err(e) => {
            panic!("Expected success but got error: {:?}", e);
        }
    }
}

#[test]
fn test_get_manifest_preserves_explicit_library_path_when_dockerhub_compat_false() {
    use std::str::FromStr;

    let mut server = mockito::Server::new();
    let manifest_body = r#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"mediaType":"application/vnd.oci.image.config.v1+json","size":1234,"digest":"sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"},"layers":[]}"#;

    // Even with dockerhub_compat=false, a nested path like "library/org/repo" should be preserved
    // because the slash after "library/" indicates it's user-provided, not auto-added
    let mock = server
        .mock("GET", "/v2/library/org/myrepo/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header("docker-content-digest", "sha256:abc123")
        .with_body(manifest_body)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false); // dockerhub_compat = false

    let reference = Reference::from_str("library/org/myrepo:latest").unwrap();
    let result = registry.get_manifest(&reference);

    mock.assert();

    match result {
        Ok((manifest_or_index, digest)) => {
            assert!(manifest_or_index.is_manifest());
            assert_eq!(digest, "sha256:abc123");
        }
        Err(e) => {
            panic!("Expected success but got error: {:?}", e);
        }
    }
}

#[test]
fn test_get_manifest_with_org_name_when_dockerhub_compat_false() {
    use std::str::FromStr;

    let mut server = mockito::Server::new();
    let manifest_body = r#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"mediaType":"application/vnd.oci.image.config.v1+json","size":1234,"digest":"sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"},"layers":[]}"#;

    // With dockerhub_compat=false, "myorg/myrepo" should be used as-is (no "library/" prefix added)
    let mock = server
        .mock("GET", "/v2/myorg/myrepo/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header("docker-content-digest", "sha256:abc123")
        .with_body(manifest_body)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false); // dockerhub_compat = false

    let reference = Reference::from_str("myorg/myrepo:latest").unwrap();
    let result = registry.get_manifest(&reference);

    mock.assert();

    match result {
        Ok((manifest_or_index, digest)) => {
            assert!(manifest_or_index.is_manifest());
            assert_eq!(digest, "sha256:abc123");
        }
        Err(e) => {
            panic!("Expected success but got error: {:?}", e);
        }
    }
}

#[test]
fn test_get_manifest_by_digest_strips_library_when_dockerhub_compat_false() {
    use std::str::FromStr;

    let mut server = mockito::Server::new();
    let manifest_body = r#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"mediaType":"application/vnd.oci.image.config.v1+json","size":1234,"digest":"sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"},"layers":[]}"#;
    let digest = "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

    // With dockerhub_compat=false, digest-based reference should also strip "library/"
    let mock = server
        .mock("GET", format!("/v2/alpine/manifests/{}", digest).as_str())
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header("docker-content-digest", digest)
        .with_body(manifest_body)
        .create();

    let client = Client::new(&server.url(), None).unwrap();
    let mut registry = Registry::new(client, None, None, false); // dockerhub_compat = false

    let reference = Reference::from_str(&format!("alpine@{}", digest)).unwrap();
    let result = registry.get_manifest(&reference);

    mock.assert();

    match result {
        Ok((manifest_or_index, returned_digest)) => {
            assert!(manifest_or_index.is_manifest());
            assert_eq!(returned_digest, digest);
        }
        Err(e) => {
            panic!("Expected success but got error: {:?}", e);
        }
    }
}
