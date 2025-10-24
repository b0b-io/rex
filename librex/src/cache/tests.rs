use super::*;
use crate::config::Config;
use std::num::NonZeroUsize;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_cache_new() {
    let temp_dir = tempdir().unwrap();
    let config = Config::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let cache = Cache::new(
        temp_dir.path().to_path_buf(),
        config.cache.ttl.clone(),
        capacity,
    );

    assert_eq!(cache.memory.cap(), capacity);
    assert_eq!(cache.disk_path, temp_dir.path());
    assert_eq!(cache.ttl_config, config.cache.ttl);
}

#[test]
fn test_cache_l1_get_and_set() {
    let temp_dir = tempdir().unwrap();
    let config = Config::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(temp_dir.path().to_path_buf(), config.cache.ttl, capacity);

    let key = "my-key";
    let data = "my-data".to_string();

    // 1. First get should be a miss
    let result: Option<String> = cache.get(key).unwrap();
    assert!(result.is_none());

    // 2. Set the data
    cache.set(key, &data, CacheType::Tags).unwrap();

    // 3. Second get should be a hit
    let result: Option<String> = cache.get(key).unwrap();
    assert_eq!(result, Some(data));
}

#[test]
fn test_cache_l2_disk_hit() {
    let temp_dir = tempdir().unwrap();
    let config = Config::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(temp_dir.path().to_path_buf(), config.cache.ttl, capacity);

    let key = "my-key";
    let data = "my-data-on-disk".to_string();

    // Manually create an L2 cache entry by calling `set`
    cache.set(key, &data, CacheType::Tags).unwrap();
    // Clear L1 to force a disk read
    cache.memory.clear();

    // Get should be a hit, and it should populate the L1 cache
    let result: Option<String> = cache.get(key).unwrap();
    assert_eq!(result, Some(data));

    // Verify L1 cache is now populated
    assert!(cache.memory.get(key).is_some());
}

#[test]
fn test_cache_l2_disk_expired() {
    let temp_dir = tempdir().unwrap();
    let mut config = Config::default();
    config.cache.ttl.tags = 1; // Set a 1-second TTL for tags
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(
        temp_dir.path().to_path_buf(),
        config.cache.ttl.clone(),
        capacity,
    );

    let key = "my-key";
    let data = "my-expired-data".to_string();

    // Set an entry that will expire
    cache.set(key, &data, CacheType::Tags).unwrap();

    // Wait for the entry to expire
    std::thread::sleep(Duration::from_secs(2));

    // Clear the L1 cache to force an L2 read
    cache.memory.clear();

    // Get should now be a miss
    let result: Option<String> = cache.get(key).unwrap();
    assert!(result.is_none());

    // The stale file should have been deleted
    let path = cache.key_to_path(key).unwrap();
    assert!(!path.exists());
}

#[test]
fn test_cache_prune() {
    let temp_dir = tempdir().unwrap();
    let mut config = Config::default();
    config.cache.ttl.tags = 1; // 1 second TTL
    config.cache.ttl.catalog = 3600; // 1 hour TTL
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(
        temp_dir.path().to_path_buf(),
        config.cache.ttl.clone(),
        capacity,
    );

    // Create one entry that will expire
    let expired_key = "expired-key";
    let expired_data = "expired-data".to_string();
    cache
        .set(expired_key, &expired_data, CacheType::Tags)
        .unwrap();

    // Create one entry that is still valid
    let valid_key = "valid-key";
    let valid_data = "valid-data".to_string();
    cache
        .set(valid_key, &valid_data, CacheType::Catalog)
        .unwrap();

    // Wait for the first entry to expire
    std::thread::sleep(Duration::from_secs(2));

    // Prune the cache
    let stats = cache.prune().unwrap();

    // Assert that the expired file is gone
    let expired_path = cache.key_to_path(expired_key).unwrap();
    assert!(!expired_path.exists());

    // Assert that the valid file is still there
    let valid_path = cache.key_to_path(valid_key).unwrap();
    assert!(valid_path.exists());

    // Assert stats are correct
    assert_eq!(stats.removed_files, 1);
    assert!(stats.reclaimed_space > 0);
}

#[test]
fn test_cache_clear() {
    let temp_dir = tempdir().unwrap();
    let config = Config::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(temp_dir.path().to_path_buf(), config.cache.ttl, capacity);

    // Add multiple entries
    cache
        .set("key1", &"data1".to_string(), CacheType::Tags)
        .unwrap();
    cache
        .set("key2", &"data2".to_string(), CacheType::Catalog)
        .unwrap();
    cache
        .set("key3", &"data3".to_string(), CacheType::Manifest)
        .unwrap();

    // Verify entries exist
    assert!(cache.memory.len() > 0);
    let key1_path = cache.key_to_path("key1").unwrap();
    let key2_path = cache.key_to_path("key2").unwrap();
    let key3_path = cache.key_to_path("key3").unwrap();
    assert!(key1_path.exists());
    assert!(key2_path.exists());
    assert!(key3_path.exists());

    // Clear the cache
    let stats = cache.clear().unwrap();

    // Assert memory cache is empty
    assert_eq!(cache.memory.len(), 0);

    // Assert disk files are removed
    assert!(!key1_path.exists());
    assert!(!key2_path.exists());
    assert!(!key3_path.exists());

    // Assert stats are correct
    assert_eq!(stats.removed_files, 3);
    assert!(stats.reclaimed_space > 0);
}

#[test]
fn test_cache_stats() {
    let temp_dir = tempdir().unwrap();
    let config = Config::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(temp_dir.path().to_path_buf(), config.cache.ttl, capacity);

    // Initially empty
    let stats = cache.stats().unwrap();
    assert_eq!(stats.disk_entries, 0);
    assert_eq!(stats.disk_size, 0);
    assert_eq!(stats.memory_entries, 0);

    // Add entries
    cache
        .set("key1", &"data1".to_string(), CacheType::Tags)
        .unwrap();
    cache
        .set("key2", &"data2".to_string(), CacheType::Catalog)
        .unwrap();
    cache
        .set("key3", &"data3".to_string(), CacheType::Manifest)
        .unwrap();

    // Check stats
    let stats = cache.stats().unwrap();
    assert_eq!(stats.disk_entries, 3);
    assert!(stats.disk_size > 0);
    assert_eq!(stats.memory_entries, 3);

    // Clear memory cache only
    cache.memory.clear();

    // Check stats again
    let stats = cache.stats().unwrap();
    assert_eq!(stats.disk_entries, 3);
    assert!(stats.disk_size > 0);
    assert_eq!(stats.memory_entries, 0);
}

#[test]
fn test_cache_clear_empty() {
    let temp_dir = tempdir().unwrap();
    let config = Config::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let mut cache = Cache::new(temp_dir.path().to_path_buf(), config.cache.ttl, capacity);

    // Clear empty cache should not fail
    let stats = cache.clear().unwrap();
    assert_eq!(stats.removed_files, 0);
    assert_eq!(stats.reclaimed_space, 0);
}

#[test]
fn test_cache_stats_empty() {
    let temp_dir = tempdir().unwrap();
    let config = Config::default();
    let capacity = NonZeroUsize::new(100).unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf(), config.cache.ttl, capacity);

    // Stats for empty cache
    let stats = cache.stats().unwrap();
    assert_eq!(stats.disk_entries, 0);
    assert_eq!(stats.disk_size, 0);
    assert_eq!(stats.memory_entries, 0);
}
