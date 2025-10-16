# Rex Library - Public API

This document describes the high-level public API for the Rex library.

## Overview

Rex provides a simple, well-documented API for interacting with OCI registries. The library is organized into two layers:

1. **High-level API** - Recommended for most users (main documentation focus)
2. **Low-level modules** - Available for experts but hidden from docs

## Quick Start

```rust
use librex::Rex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a registry
    let mut rex = Rex::connect("http://localhost:5000").await?;

    // List repositories
    let repos = rex.list_repositories().await?;

    // Search repositories
    let results = rex.search_repositories("alpine").await?;

    Ok(())
}
```

## Main Types

### `Rex`

The primary entry point for all registry operations.

**Methods:**
- `connect(url)` - Connect to a registry with defaults
- `builder()` - Create a builder for advanced configuration
- `check()` - Verify registry is accessible
- `login(credentials)` - Set authentication credentials
- `logout()` - Clear credentials
- `list_repositories()` - List all repositories
- `list_tags(repository)` - List tags for a repository
- `get_manifest(image)` - Get manifest for an image
- `get_blob(repository, digest)` - Get a blob by digest
- `search_repositories(query)` - Fuzzy search repositories
- `search_tags(repository, query)` - Fuzzy search tags
- `search_images(query)` - Fuzzy search images (repo:tag)

### `RexBuilder`

Builder for advanced configuration with caching, authentication, etc.

**Methods:**
- `new()` - Create a new builder
- `registry_url(url)` - Set registry URL
- `with_cache(dir)` - Enable caching
- `with_config(config)` - Set configuration
- `with_config_file(path)` - Load config from file
- `with_credentials(creds)` - Set authentication
- `build()` - Build the Rex instance

**Example:**
```rust
let mut rex = Rex::builder()
    .registry_url("http://localhost:5000")
    .with_cache("/tmp/rex-cache")
    .build()
    .await?;
```

### `SearchResult`

A search result with relevance scoring.

**Fields:**
- `value: String` - The matched string
- `score: u32` - Relevance score (higher is better)

### `Credentials`

Authentication credentials for registries.

**Variants:**
- `Basic { username, password }` - HTTP Basic authentication
- `Bearer { token }` - Bearer token authentication

### `Reference`

Parsed image reference (repository:tag or repository@digest).

**Methods:**
- `parse(s)` / `from_str(s)` - Parse a reference string
- `repository()` - Get the repository name
- `tag()` - Get the tag (if any)
- `digest()` - Get the digest (if any)
- `registry()` - Get the registry (if specified)

### `Digest`

Content digest validation and handling.

**Methods:**
- `parse(s)` / `from_str(s)` - Parse a digest string
- `algorithm()` - Get the algorithm (sha256, sha512)
- `encoded()` - Get the hex-encoded hash
- `verify(data)` - Verify data matches digest

### `Config`

Configuration settings for Rex.

**Methods:**
- `default()` - Create default configuration
- `from_yaml_str(yaml)` - Parse from YAML string
- Load/save from files

## Examples

### Basic Connection

```rust
let mut rex = Rex::connect("http://localhost:5000").await?;
let repos = rex.list_repositories().await?;
```

### With Authentication

```rust
let mut rex = Rex::connect("https://registry.example.com").await?;

rex.login(Credentials::Basic {
    username: "user".to_string(),
    password: "pass".to_string(),
});

let repos = rex.list_repositories().await?;
```

### With Caching

```rust
let mut rex = Rex::builder()
    .registry_url("http://localhost:5000")
    .with_cache("/tmp/rex-cache")
    .build()
    .await?;

// Subsequent calls will use cache
let repos1 = rex.list_repositories().await?;
let repos2 = rex.list_repositories().await?; // From cache
```

### Searching

```rust
// Search repositories
let results = rex.search_repositories("alp").await?;
for result in results {
    println!("{} (score: {})", result.value, result.score);
}

// Search tags within a repository
let results = rex.search_tags("alpine", "3.").await?;

// Search images (repository:tag)
let results = rex.search_images("alp:lat").await?;
// Returns: alpine:latest, etc.
```

### Getting Manifests

```rust
let manifest = rex.get_manifest("alpine:latest").await?;
println!("Layers: {}", manifest.layers().len());

// Can also use digest references
let manifest = rex.get_manifest("alpine@sha256:abc123...").await?;
```

## Low-Level Modules

For advanced users who need fine-grained control, all low-level modules are available:

- `librex::client` - HTTP client for registry communication
- `librex::auth` - Authentication handling
- `librex::registry` - Registry operations
- `librex::cache` - Caching layer
- `librex::config` - Configuration management
- `librex::search` - Fuzzy search algorithms
- `librex::oci` - OCI specification types
- `librex::reference` - Image reference parsing
- `librex::digest` - Digest validation
- `librex::format` - Data formatting utilities
- `librex::error` - Error types

These modules are marked `#[doc(hidden)]` but are still fully accessible for experts.

## Error Handling

All operations return `Result<T, RexError>` where `RexError` provides detailed error information:

```rust
match rex.list_repositories().await {
    Ok(repos) => println!("Found {} repositories", repos.len()),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Running Examples

```bash
# Basic usage example
cargo run --example basic_usage
```

## Documentation

Generate and view the full API documentation:

```bash
cargo doc --open
```

The documentation will show the high-level API prominently, with low-level modules hidden by default.
