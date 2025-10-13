# Cache Module Notes

## 1. Architecture & Core Principles

- **Goal**: To improve performance and responsiveness by reducing network requests. The cache serves as a local data source for the `registry` module.
- **Strategy**: A two-tier cache system:
  - **L1 (Memory)**: An in-memory LRU (Least Recently Used) cache for extremely fast access to hot data within a single application session.
  - **L2 (Disk)**: A persistent on-disk cache to speed up subsequent application runs.

## 2. Library Decisions

- **L1 (Memory)**: We will use the `lru` crate.
  - **Rationale**: It is the lightweight, community standard for in-memory LRU caching in Rust.
- **L2 (Disk Serialization)**: We will use the `bincode` crate.
  - **Rationale**: Bincode provides extremely fast, compact binary serialization. Since the cache is an internal component for a Rust-only application, the portability of formats like JSON is not needed, and Bincode's performance is the top priority.

## 3. Data Structures & API

### `CacheType` Enum

This enum allows the caller to specify the *type* of data being cached, decoupling them from the specific TTL implementation.

```rust
pub enum CacheType {
    Catalog,
    Tags,
    Manifest,
    Config,
}
```

### `Cache` Struct

The `Cache` is initialized with the application's TTL configuration, making it self-contained.

```rust
pub struct Cache {
    memory: LruCache<String, Vec<u8>>,
    disk_path: PathBuf,
    ttl_config: CacheTtl, // <-- From the `config` module
}
```

### `set` Method

The `set` method uses the `CacheType` to look up the correct TTL from its internal `ttl_config`.

```rust
pub fn set<T: Serialize>(&mut self, key: &str, data: &T, cache_type: CacheType) -> Result<()>
```

## 4. Lifecycle and Interaction with `registry` Module

The data flow remains the same as previously discussed, but the responsibility for choosing a TTL is now entirely within the `cache` module.

1.  **Request**: `registry` module calls `cache.get(key)`.
2.  **Cache Lookup**: `cache` module checks L1, then L2, returning data if a fresh entry is found.
3.  **Network Fetch**: On a cache miss (`None`), the `registry` module calls the `client` module.
4.  **Cache Hydration**: After a successful network fetch, the `registry` module calls `cache.set(key, &data, CacheType::Tags)`, telling the cache *what* was fetched, not for how long to cache it.
5.  **Cache Write**: The `cache` module looks up the TTL for `CacheType::Tags` in its `ttl_config`, creates the `CacheEntry`, and writes it to both L1 and L2 caches.