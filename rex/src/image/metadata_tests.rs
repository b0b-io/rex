//! Tests for tag and repository metadata fetching.

use super::*;

#[test]
fn test_tag_metadata_fetcher_new_creates_instance() {
    let fetcher = TagMetadataFetcher::new(
        "localhost:5000".to_string(),
        Path::new("/tmp/cache"),
        None,
        8,
    );

    assert_eq!(fetcher.registry_url, "localhost:5000");
    assert_eq!(fetcher.concurrency, 8);
}

#[test]
fn test_repository_metadata_fetcher_new_creates_instance() {
    let fetcher = RepositoryMetadataFetcher::new(
        "localhost:5000".to_string(),
        Path::new("/tmp/cache"),
        None,
        8,
    );

    assert_eq!(fetcher.registry_url, "localhost:5000");
    assert_eq!(fetcher.concurrency, 8);
}

// Integration tests would require a running registry
// These are placeholder tests for now
