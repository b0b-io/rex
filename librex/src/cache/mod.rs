//! Two-tier caching system (L1: Memory, L2: Disk).
//!
//! This module provides a cache that serves as a fast local data source
//! for the `registry` module, reducing network requests.

use crate::config::CacheTtl;
use crate::error::{Result, RexError};
use bincode::{Decode, Encode, config::standard};
use lru::LruCache;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

#[cfg(test)]
mod tests;

/// The type of data being cached, used to determine the correct TTL.
#[derive(Clone, Copy)]
pub enum CacheType {
    Catalog,
    Tags,
    Manifest,
    Config,
}

/// A wrapper for cached data that includes metadata for expiration.
/// This is the format that is serialized to disk.
#[derive(Serialize, Deserialize, Encode, Decode)]
struct CacheEntry {
    /// The raw, serialized data.
    data: Vec<u8>,
    /// The UTC timestamp of when the entry was cached.
    cached_at: SystemTime,
    /// The duration for which this entry is considered valid.
    ttl: Duration,
}

/// Statistics returned after a prune operation.
#[derive(Debug, Default)]
pub struct PruneStats {
    /// The number of expired files removed.
    pub removed_files: u64,
    /// The total disk space reclaimed in bytes.
    pub reclaimed_space: u64,
}

/// Statistics about the cache.
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Total number of entries in disk cache.
    pub disk_entries: u64,
    /// Total size of disk cache in bytes.
    pub disk_size: u64,
    /// Number of entries in memory cache.
    pub memory_entries: u64,
}

/// Statistics returned after a clear operation.
#[derive(Debug, Default)]
pub struct ClearStats {
    /// The number of files removed.
    pub removed_files: u64,
    /// The total disk space reclaimed in bytes.
    pub reclaimed_space: u64,
}

/// Manages the L1 (memory) and L2 (disk) caches.
pub struct Cache {
    /// The L1 in-memory cache.
    memory: LruCache<String, Vec<u8>>,
    /// The base path on the filesystem for the L2 disk cache.
    disk_path: PathBuf,
    /// The TTL configuration for different cache types.
    ttl_config: CacheTtl,
}

impl Cache {
    /// Creates a new `Cache`.
    pub fn new(disk_path: PathBuf, ttl_config: CacheTtl, memory_capacity: NonZeroUsize) -> Self {
        Self {
            memory: LruCache::new(memory_capacity),
            disk_path,
            ttl_config,
        }
    }

    /// Retrieves an entry from the cache.
    pub fn get<T: DeserializeOwned + Encode + Decode<()>>(
        &mut self,
        key: &str,
    ) -> Result<Option<T>> {
        // L1 Check
        if let Some(bytes) = self.memory.get(key) {
            let (entry, _): (CacheEntry, usize) = bincode::decode_from_slice(bytes, standard())
                .map_err(|e| {
                    RexError::validation_with_source("Failed to deserialize L1 cache entry", e)
                })?;

            if SystemTime::now()
                .duration_since(entry.cached_at)
                .unwrap_or_default()
                <= entry.ttl
            {
                let (data, _): (T, usize) = bincode::decode_from_slice(&entry.data, standard())
                    .map_err(|e| {
                        RexError::validation_with_source("Failed to deserialize L1 data", e)
                    })?;
                return Ok(Some(data));
            } else {
                self.memory.pop(key);
            }
        }

        // L2 Check
        let path = self.key_to_path(key)?;
        if !path.exists() {
            return Ok(None);
        }

        let bytes = std::fs::read(&path).map_err(|e| {
            RexError::config_with_source(
                "Failed to read L2 cache file",
                Some(path.display().to_string()),
                e,
            )
        })?;

        let (entry, _): (CacheEntry, usize) = bincode::decode_from_slice(&bytes, standard())
            .map_err(|e| {
                RexError::validation_with_source("Failed to deserialize L2 cache entry", e)
            })?;

        if SystemTime::now()
            .duration_since(entry.cached_at)
            .unwrap_or_default()
            > entry.ttl
        {
            let _ = std::fs::remove_file(path);
            return Ok(None);
        }

        // Hydrate L1 cache
        self.memory.put(key.to_string(), bytes);

        let (data, _): (T, usize) = bincode::decode_from_slice(&entry.data, standard())
            .map_err(|e| RexError::validation_with_source("Failed to deserialize L2 data", e))?;
        Ok(Some(data))
    }

    /// Adds or updates an entry in the cache.
    pub fn set<T: Serialize + Encode + Decode<()>>(
        &mut self,
        key: &str,
        data: &T,
        cache_type: CacheType,
    ) -> Result<()> {
        let ttl = self.get_ttl(cache_type);
        let data_bytes = bincode::encode_to_vec(data, standard())
            .map_err(|e| RexError::validation_with_source("Failed to serialize cache data", e))?;

        let entry = CacheEntry {
            data: data_bytes,
            cached_at: SystemTime::now(),
            ttl,
        };

        let entry_bytes = bincode::encode_to_vec(&entry, standard())
            .map_err(|e| RexError::validation_with_source("Failed to serialize cache entry", e))?;

        // L2 Write
        let path = self.key_to_path(key)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RexError::config_with_source(
                    "Failed to create cache directory",
                    Some(parent.display().to_string()),
                    e,
                )
            })?;
        }
        std::fs::write(&path, &entry_bytes).map_err(|e| {
            RexError::config_with_source(
                "Failed to write L2 cache file",
                Some(path.display().to_string()),
                e,
            )
        })?;

        // L1 Write
        self.memory.put(key.to_string(), entry_bytes);

        Ok(())
    }

    /// Deletes a specific entry from the cache (both L1 and L2).
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` regardless of whether the key existed in the cache.
    /// Errors are only returned if there are issues accessing the filesystem.
    pub fn delete(&mut self, key: &str) -> Result<()> {
        // Remove from L1 memory cache
        self.memory.pop(key);

        // Remove from L2 disk cache
        let path = self.key_to_path(key)?;
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| {
                RexError::config_with_source(
                    "Failed to delete cache file",
                    Some(path.display().to_string()),
                    e,
                )
            })?;
        }

        Ok(())
    }

    /// Removes expired files from the on-disk cache.
    pub fn prune(&self) -> Result<PruneStats> {
        let mut stats = PruneStats::default();
        if !self.disk_path.exists() {
            return Ok(stats);
        }

        for entry in WalkDir::new(&self.disk_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if let Ok(bytes) = std::fs::read(path) {
                match bincode::decode_from_slice::<CacheEntry, _>(&bytes, standard()) {
                    Ok((cached_entry, _)) => {
                        if SystemTime::now()
                            .duration_since(cached_entry.cached_at)
                            .unwrap_or_default()
                            > cached_entry.ttl
                        {
                            if let Ok(metadata) = std::fs::metadata(path) {
                                stats.reclaimed_space += metadata.len();
                            }
                            if std::fs::remove_file(path).is_ok() {
                                stats.removed_files += 1;
                            }
                        }
                    }
                    Err(_) => {
                        // Ignore deserialization errors
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Clears all cache entries from disk and memory.
    pub fn clear(&mut self) -> Result<ClearStats> {
        let mut stats = ClearStats::default();

        // Clear L1 memory cache
        self.memory.clear();

        // Clear L2 disk cache
        if !self.disk_path.exists() {
            return Ok(stats);
        }

        for entry in WalkDir::new(&self.disk_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if let Ok(metadata) = std::fs::metadata(path) {
                stats.reclaimed_space += metadata.len();
            }
            if std::fs::remove_file(path).is_ok() {
                stats.removed_files += 1;
            }
        }

        Ok(stats)
    }

    /// Gets statistics about the cache.
    pub fn stats(&self) -> Result<CacheStats> {
        let mut stats = CacheStats {
            memory_entries: self.memory.len() as u64,
            ..Default::default()
        };

        // Walk disk cache
        if !self.disk_path.exists() {
            return Ok(stats);
        }

        for entry in WalkDir::new(&self.disk_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if let Ok(metadata) = std::fs::metadata(path) {
                stats.disk_entries += 1;
                stats.disk_size += metadata.len();
            }
        }

        Ok(stats)
    }

    /// Converts a cache key into a full filesystem path.
    fn key_to_path(&self, key: &str) -> Result<PathBuf> {
        if key.contains("..") || key.starts_with('/') {
            return Err(RexError::validation("Invalid cache key format"));
        }
        Ok(self.disk_path.join(key))
    }

    /// Gets the correct TTL `Duration` for a given `CacheType`.
    fn get_ttl(&self, cache_type: CacheType) -> Duration {
        let seconds = match cache_type {
            CacheType::Catalog => self.ttl_config.catalog,
            CacheType::Tags => self.ttl_config.tags,
            CacheType::Manifest => self.ttl_config.manifest,
            CacheType::Config => self.ttl_config.config,
        };
        Duration::from_secs(seconds)
    }
}
