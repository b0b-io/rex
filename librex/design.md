# librex - Library Design

## Overview

librex is a Rust library that provides OCI-compliant container registry
interaction functionality. It is completely UI-agnostic and can be used by CLI
tools, TUI applications, or embedded in other Rust projects.

## Architecture

librex is the core engine that powers the rex CLI and TUI. It handles all
registry operations, authentication, caching, and data management.

```text
┌─────────────────────────────────────────┐
│         Rex CLI / TUI                    │
│    (rex crate - frontends)              │
└─────────────────────────────────────────┘
              │
              │  uses
              ▼
┌─────────────────────────────────────────┐
│       librex (Core Library)              │
│  - Registry client                      │
│  - Authentication                       │
│  - OCI operations                       │
│  - Caching (L1 + L2)                    │
│  - Search & filtering                   │
│  - Data models                          │
└─────────────────────────────────────────┘
              │
              │  calls
              ▼
┌─────────────────────────────────────────┐
│    OCI Distribution Specification       │
│         (HTTP/REST API)                  │
└─────────────────────────────────────────┘
```

## Part 1: Core Engine Design (librex)

The core engine is implemented as a Rust library crate that provides all
registry interaction functionality. It is completely UI-agnostic and can be
used by both CLI and TUI interfaces.

### Module Structure

```text
librex/
├── client/           # HTTP client and registry communication
├── auth/             # Authentication handling
├── registry/         # Registry operations
├── oci/              # OCI specification data models
├── reference/        # Image reference parsing
├── digest/           # Digest validation and handling
├── search/           # Fuzzy search and filtering
├── config/           # Configuration management
├── cache/            # Caching layer
├── format/           # Data formatting (size, timestamps)
└── error/            # Error types
```

### 1.1 Client Module (`client/`)

Handles all HTTP communication with OCI registries.

#### HTTP Client Component

**Purpose**: Manage low-level HTTP requests to registry endpoints

**Responsibilities**:

- Execute HTTP requests (GET, HEAD, PUT for OCI Distribution Specification)
- Manage connection pooling for performance
- Handle timeouts and automatic retries on transient failures
- Support both HTTP and HTTPS with configurable TLS verification
- Allow custom CA certificates for private registries
- Provide request/response logging for debugging
- Set custom User-Agent headers

**Configuration**:

- Request timeout duration
- Maximum retry attempts
- TLS verification enabled/disabled
- Custom CA certificate paths
- User agent string

#### Registry Endpoint Component

**Purpose**: Represent and validate registry URLs

**Responsibilities**:

- Parse registry URLs into components (scheme, host, port)
- Validate URL format and structure
- Normalize URLs (add default ports, clean up paths)
- Generate OCI Distribution Specification v2 API base URLs
- Handle default registry assumptions (localhost:5000 for Zot)

**URL Components**:

- Scheme: HTTP or HTTPS
- Host: Domain or IP address
- Port: Optional (defaults: 80 for HTTP, 443 for HTTPS, 5000 for localhost)

### 1.2 Authentication Module (`auth/`)

Handles all authentication mechanisms for OCI registries according to the
OCI Distribution Specification authentication flow.

#### Authentication Methods

**Anonymous Access**:

- No credentials required
- Used for public registries or registries without authentication

**Basic Authentication**:

- Username and password
- Sent via HTTP Basic Auth header
- Used for simple authentication schemes

**Bearer Token Authentication**:

- OAuth2-style token authentication
- Follows WWW-Authenticate challenge/response flow
- Token obtained from authentication service
- Token cached and reused until expiration

#### Authentication Manager Component

**Purpose**: Orchestrate authentication flow and credential management

**Responsibilities**:

- Store credentials for multiple registries
- Implement WWW-Authenticate challenge/response protocol:
  1. Attempt request without auth
  2. Receive 401 with WWW-Authenticate header
  3. Parse authentication realm, service, and scope
  4. Request token from auth service
  5. Retry original request with Bearer token
- Cache tokens to avoid repeated authentication
- Handle token expiration and refresh
- Generate appropriate headers for authenticated requests

**Token Caching**:

- Store tokens by registry and scope
- Track expiration time if provided
- Automatically refresh expired tokens
- Clear cache on authentication failures

#### Credential Store Component

**Purpose**: Load and store registry credentials from various sources

**Credential Sources** (in priority order):

1. Docker config file (`~/.docker/config.json`)
2. Podman auth file (`~/.config/containers/auth.json`)
3. OS-specific secure storage (keychain/keyring)
4. Interactive prompt

**Responsibilities**:

- Parse Docker/Podman credential formats
- Decode base64-encoded credentials
- Store new credentials securely in OS keychain
- Support interactive credential prompts
- Handle credential deletion (logout)

**Security Considerations**:

- Never store plaintext passwords in config files
- Use OS keychain/credential manager when available (macOS Keychain,
  Linux Secret Service, Windows Credential Manager)
- Mask passwords in logs and error messages
- Clear sensitive data from memory after use

### 1.3 Registry Module (`registry/`)

Implements OCI Distribution Specification operations.

#### Registry Client Component

**Purpose**: High-level interface for all registry operations

**Responsibilities**:

- Orchestrate HTTP client and authentication manager
- Implement OCI Distribution Specification v2 API endpoints
- Handle pagination for large result sets
- Parse and validate responses
- Integrate with optional caching layer
- Provide error context and recovery

#### Supported Operations

**Version Check**:

- Endpoint: `GET /v2/`
- Verifies registry supports OCI Distribution Specification
- Returns API version information

**List Repositories** (Catalog):

- Endpoint: `GET /v2/_catalog`
- Lists all repositories in the registry
- Supports pagination with `n` (limit) and `last` parameters
- Returns repository names as array

**List Tags**:

- Endpoint: `GET /v2/<name>/tags/list`
- Lists all tags for a specific repository
- Supports pagination
- Returns tag names as array

**Get Manifest**:

- Endpoint: `GET /v2/<name>/manifests/<reference>`
- Retrieves manifest by tag or digest
- Supports content negotiation via Accept header
- Returns manifest in requested format
- Includes Docker-Content-Digest header

**Check Manifest Existence**:

- Endpoint: `HEAD /v2/<name>/manifests/<reference>`
- Checks if manifest exists without downloading
- Returns digest in Docker-Content-Digest header

**Get Blob**:

- Endpoint: `GET /v2/<name>/blobs/<digest>`
- Retrieves blob content (config, layers)
- Verifies digest after download
- Handles redirects to CDN or storage backend

**Check Blob Existence**:

- Endpoint: `HEAD /v2/<name>/blobs/<digest>`
- Checks if blob exists without downloading

#### Pagination Handling

**Concept**: Large registries may have thousands of repositories/tags

**Approach**:

- Use `n` parameter to limit results per request
- Use `last` parameter to continue from previous result
- Parse `Link` header for next page URL
- Provide iterator-style interface for consumers
- Allow callers to specify page size or fetch all

**Example Flow**:

1. Request: `GET /v2/_catalog?n=100`
2. Response: 100 repos + Link header pointing to next page
3. Extract `last` value from response
4. Request: `GET /v2/_catalog?n=100&last=<last_repo>`
5. Repeat until no Link header

### 1.4 OCI Module (`oci/`)

Data structures representing OCI specification types. All structures map
directly to OCI Image Spec and OCI Distribution Specification formats.

#### Manifest Types

**Manifest Enum**:

- Top-level type that can represent any manifest format
- Variants:
  - OCI Image Manifest
  - OCI Image Index (multi-platform)
  - Docker Image Manifest V2 Schema 2
  - Docker Manifest List

**Common Operations**:

- Get media type
- Calculate/verify digest
- Get total size
- Extract config reference (for image manifests)
- Extract layer list (for image manifests)
- Extract platform manifests (for indexes/lists)

#### OCI Image Manifest

Represents a single image for a specific platform.

**Fields**:

- Schema version (always 2)
- Media type
- Config descriptor (points to image config blob)
- Layers array (ordered list of filesystem layers)
- Annotations (optional key-value metadata)

#### OCI Image Index

Represents a multi-platform image (manifest list).

**Fields**:

- Schema version (always 2)
- Media type
- Manifests array (list of platform-specific manifests)
- Annotations (optional metadata)

#### Descriptor

References content by digest (manifest or blob).

**Fields**:

- Media type (identifies content type)
- Digest (SHA256 hash)
- Size (bytes)
- URLs (optional alternate locations)
- Annotations (optional metadata)
- Platform (optional, for index entries)

#### Platform

Describes OS/architecture for platform-specific images.

**Fields**:

- Architecture (amd64, arm64, arm, etc.)
- OS (linux, windows, darwin, etc.)
- OS version (optional, for Windows)
- OS features (optional)
- Variant (optional, for ARM versions like v6, v7, v8)

#### Image Configuration

The config blob referenced by image manifests.

**Fields**:

- Architecture and OS
- Container config (runtime configuration):
  - User
  - Exposed ports
  - Environment variables
  - Entrypoint
  - Command (Cmd)
  - Volumes
  - Working directory
  - Labels
- Root filesystem:
  - Type (usually "layers")
  - Diff IDs (uncompressed layer digests)
- History (layer creation history)
- Created timestamp
- Author

#### Media Type Handling

**Purpose**: Identify and work with different content types

**Responsibilities**:

- Recognize OCI media types
- Recognize Docker media types
- Determine if media type is manifest vs blob
- Determine if manifest is single-image vs index
- Generate Accept headers for content negotiation

**Supported Media Types**:

- `application/vnd.oci.image.manifest.v1+json`
- `application/vnd.oci.image.index.v1+json`
- `application/vnd.docker.distribution.manifest.v2+json`
- `application/vnd.docker.distribution.manifest.list.v2+json`
- `application/vnd.oci.image.config.v1+json`
- `application/vnd.oci.image.layer.v1.tar+gzip`

### 1.5 Reference Module (`reference/`)

Parsing and handling of image references.

#### Image Reference Component

**Purpose**: Parse and manipulate image reference strings

**Format**: `[registry/]repository[:tag|@digest]`

**Components**:

- Registry: Optional domain/IP:port
- Repository: Required name (may include namespace/path)
- Reference: Either tag (string) or digest (sha256:...)

**Parsing Rules**:

- If no registry specified, use default registry
- If no tag or digest specified, assume "latest" tag
- Repository may contain multiple path segments (e.g., `library/alpine`, `myorg/team/app`)
- Registry must include explicit port if non-standard
- Digest must be prefixed with algorithm (e.g., `sha256:`)

**Examples**:

- `myrepo:latest` → default registry, repository "myrepo", tag "latest"
- `localhost:5000/myrepo:v1` → registry "localhost:5000", repository "myrepo", tag "v1"
- `ghcr.io/user/repo@sha256:abc123...` → registry "ghcr.io", repository "user/repo", digest reference

**Validation**:

- Registry hostname follows DNS rules
- Repository name follows OCI Distribution Specification naming rules
- Tag names are valid (alphanumeric, `.`, `_`, `-`)
- Digest format is valid

**Normalization**:

- Convert to canonical form
- Add default registry if missing
- Add default tag if missing
- Ensure consistent formatting

### 1.6 Digest Module (`digest/`)

Validation and manipulation of content digests.

#### Digest Component

**Purpose**: Work with content-addressable digests

**Format**: `algorithm:encoded`

**Supported Algorithms**:

- SHA256 (required by OCI spec, most common)
- SHA512 (optional, supported but rarely used)

**Responsibilities**:

- Parse digest strings
- Validate digest format
- Validate hex encoding
- Verify content against digest (compute hash and compare)
- Convert between digest representations

**Validation Rules**:

- Algorithm must be recognized (sha256, sha512)
- Encoded part must be valid hex
- Length must match algorithm (64 chars for SHA256, 128 for SHA512)
- Format must be `algorithm:hex`

**Use Cases**:

- Verify downloaded blobs match expected digest
- Generate cache keys based on digests
- Compare manifests by digest
- Content-addressable storage

### 1.7 Search Module (`search/`)

Provides fuzzy search and filtering capabilities for repositories, images, and tags.

#### Search Component

**Purpose**: Enable fuzzy searching across registry data

**Responsibilities**:

- Perform fuzzy matching on strings (repository names, tags)
- Rank results by relevance
- Support multiple search algorithms
- Filter results based on patterns
- Handle case-insensitive matching
- Support substring matching

#### Fuzzy Matching Algorithm

**Approach**: Use approximate string matching algorithms

**Supported Algorithms**:

1. **Substring matching**: Simple, fast for exact substring queries
2. **Levenshtein distance**: Edit distance for typo tolerance
3. **Fuzzy matching**: Character-by-character matching (similar to fzf, vim-ctrl-p)
   - Matches if query characters appear in order in target string
   - Does not require contiguous match
   - Example: "alp" matches "alpine", "app" matches "myapp"

**Default**: Fuzzy matching (most flexible and user-friendly)

#### Scoring and Ranking

**Relevance Score Factors**:

1. **Match type**:
   - Exact match: highest score
   - Prefix match: high score
   - Fuzzy match: scored by distance and position
2. **Match position**:
   - Earlier matches score higher
   - Matches at word boundaries score higher
3. **Match density**:
   - Shorter target strings with matches score higher
   - Contiguous character matches score higher
4. **Case sensitivity**:
   - Case-sensitive matches score higher than case-insensitive

**Result Ordering**:

- Sort by relevance score (descending)
- Only include matches with >= 50% accuracy score
- Secondary sort by alphabetical order for equal scores
- Configurable maximum results
- Scores are calculated but not displayed to users

#### Search Types

**Repository Search**:

- Input: List of repository names from catalog
- Query: User search string
- Output: Filtered and ranked list of matching repositories

**Tag Search**:

- Input: List of tags for a repository
- Query: User search string
- Output: Filtered and ranked list of matching tags

**Image Search** (Combined):

- Search across both repository names and tags
- Format results as `repository:tag`
- Support filtering by repository first, then tag

#### Filtering

**Pattern Matching**:

- Support wildcards (`*`, `?`)
- Support regular expressions
- Support exact string matching (with quotes)

**Examples**:

- `alpine` → fuzzy match "alpine"
- `"alpine"` → exact match "alpine"
- `alpine*` → wildcard, starts with "alpine"
- `^alpine$` → regex, exact match

#### Performance Considerations

**Optimization for Large Datasets**:

- Use early termination for low-scoring matches
- Implement result limit to avoid processing all items
- Consider indexing for very large registries (future enhancement)
- Cache search results for repeated queries

**In-Memory Search**:

- Search operates on already-fetched data
- No additional registry API calls
- Fast and responsive for interactive use (TUI)

#### Integration with Registry Operations

**Workflow**:

1. Fetch repositories/tags from registry (via registry client)
2. Pass results to search module
3. Apply fuzzy search with user query
4. Return ranked results
5. Display to user (CLI/TUI)

**Use Cases**:

- CLI: `rex search image <query>` - search repositories by name
- CLI: `rex search image <query> --tag <tag>` - search repositories with specific
  tag
- CLI: `rex search tags <query>` - search tags across all repositories
- CLI: `rex search tags <query> --image <name>` - search tags within specific
  image
- TUI: Real-time filtering as user types in search box
- TUI: Incremental search while browsing

### 1.8 Configuration Module (`config/`)

Manages application configuration with sensible defaults.

#### Configuration Structure

**Output Settings**:

- Default output format (pretty/json/yaml)
- Color output behavior (auto/always/never)
- Quiet mode (suppress visual feedback)

**Network Settings**:

- Request timeout (default: 30 seconds)
- Retry attempts (default: 3)
- TLS verification enabled/disabled
- User agent string

**Cache Settings**:

- Enable/disable caching
- Cache TTL per type (catalog, tags, manifest, config)
- Cache limits (memory/disk entries and size)
- Cache behavior (consistency level, serve stale)

**TUI Settings**:

- Color theme (dark/light)
- Enable vim keybindings
- Preferred panel layout

**Registry Management**:

- List of configured registries (name, URL, connection settings)
- Current/default registry selection
- Per-registry settings (insecure, skip TLS verification)

#### Configuration File

**Location**: `~/.config/rex/config.toml`

**Format**: TOML (human-readable, easy to edit)

**Loading Priority**:

1. Command-line flags (highest priority)
2. Environment variables
3. Config file
4. Built-in defaults (lowest priority)

**Example Structure**:

```toml
[output]
format = "pretty"
color = "auto"

[network]
timeout = 30
retry_attempts = 3
verify_tls = true

[cache]
enabled = true

[cache.ttl]
catalog = 300
tags = 300
manifest = 86400
config = 86400

[tui]
theme = "dark"
vim_bindings = true

[registries]
current = "local"

[[registries.list]]
name = "local"
url = "http://localhost:5000"
insecure = true

[[registries.list]]
name = "prod"
url = "https://registry.example.com"
insecure = false
```

### 1.9 Cache Module (`cache/`)

Persistent caching layer for registry responses to improve performance and
reduce network requests across CLI invocations.

#### Cache Architecture

**Write-Through Cache**:

- Two-tier architecture: Memory (L1) + Disk (L2)
- Writes go to both memory and disk simultaneously
- Reads check memory first, then disk, then registry
- Memory cache is a subset of disk cache (hot entries)

**Purpose**:

- CLI: Persist cache across invocations for faster repeated operations
- TUI: Use memory cache during session, persist to disk on exit
- Share cache data between CLI and TUI modes

#### Cache Storage Layers

**Memory Layer (L1 Cache)**:

- In-process hash map
- Fast access (nanoseconds)
- Limited size (default: 1000 entries or 100MB)
- LRU eviction when capacity reached
- Populated from disk on demand (lazy loading)
- Cleared on process exit

**Disk Layer (L2 Cache)**:

- Persistent storage on filesystem
- Location: `~/.cache/rex/` (XDG Base Directory compliant)
- Format: Individual JSON files per cache entry
- Slower access (milliseconds) but persistent
- Larger capacity (default: 10000 entries or 1GB)
- Survives process restarts

#### Cache Storage Format

**Directory Structure**:

```text
~/.cache/rex/
├── catalogs/
│   └── {registry_hash}/
│       └── catalog.json
├── tags/
│   └── {registry_hash}/
│       └── {repo_hash}/
│           └── tags.json
├── manifests/
│   └── {registry_hash}/
│       └── {repo_hash}/
│           └── {digest}.json
├── configs/
│   └── {registry_hash}/
│       └── {repo_hash}/
│           └── {digest}.json
└── metadata.json  # Cache-wide metadata (stats, last cleanup)
```

**File Format**:

- Each cache entry is a JSON file
- Includes data + metadata (timestamp, TTL, size)
- Atomic writes using temp file + rename pattern
- Includes version field for schema evolution

**Example Entry**:

```json
{
  "version": 1,
  "cached_at": "2024-01-15T10:30:00Z",
  "ttl_seconds": 300,
  "registry": "localhost:5000",
  "key": "catalog",
  "data": {
    "repositories": ["repo1", "repo2", "repo3"]
  }
}
```

#### What to Cache

**Cacheable Data**:

1. **Repository Catalog**:
   - TTL: 5 minutes (configurable)
   - Mutable: repositories can be added/removed
   - Key: `{registry}/catalog`

2. **Tag Lists**:
   - TTL: 5 minutes (configurable)
   - Mutable: tags can be added/removed
   - Key: `{registry}/{repository}/tags`

3. **Manifests** (by digest):
   - TTL: 24 hours or infinite (configurable)
   - Immutable: digest-addressed content never changes
   - Key: `{registry}/{repository}/manifests/{digest}`

4. **Image Configs** (by digest):
   - TTL: 24 hours or infinite (configurable)
   - Immutable: digest-addressed content never changes
   - Key: `{registry}/{repository}/configs/{digest}`

**Non-Cacheable Data**:

- Tag-to-digest resolution (tags are mutable pointers)
- Authentication tokens (handled by auth module)
- Blob content (too large, not useful for CLI)

#### Cache Coherence Strategy

**Staleness Detection**:

1. **Time-Based (TTL)**:
   - Each entry has creation timestamp and TTL
   - On read, check if entry is expired
   - Expired entries trigger refresh from registry
   - Option to serve stale data while refreshing

2. **Versioning** (future enhancement):
   - Track registry-provided ETag headers
   - Validate cached data with conditional HEAD requests
   - Requires additional network round-trip

**Coherence Levels**:

1. **Weak Consistency** (default):
   - Use cached data until TTL expires
   - No validation with registry
   - Fast, suitable for most CLI operations
   - Good for high-latency connections

2. **Strong Consistency**:
   - Always validate with registry before serving
   - Use conditional requests (If-None-Match with ETags)
   - Slower but guarantees fresh data
   - Activated with `--no-cache` or `--fresh` flag

3. **Eventual Consistency**:
   - Serve stale data immediately
   - Trigger background refresh
   - Update cache asynchronously
   - Good for interactive use (TUI)

#### Cache Eviction Strategies

**Entry-Level Eviction** (per-entry TTL):

- Each entry expires independently based on TTL
- Expired entries deleted on next cache cleanup
- Cleanup runs on startup and periodically

**Memory Cache Eviction** (LRU):

- When memory cache reaches size limit
- Remove least recently used entries
- Disk cache remains intact
- Ensures bounded memory usage

**Disk Cache Eviction**:

1. **Size-Based**:
   - Track total cache size
   - When limit exceeded (default: 1GB), evict oldest entries
   - Prioritize keeping immutable entries (manifests) over mutable (catalogs)

2. **Time-Based**:
   - Remove entries not accessed in X days (default: 30 days)
   - Track last access time in entry metadata
   - Prevents indefinite growth

3. **Count-Based**:
   - Limit total number of entries (default: 10000)
   - Evict oldest when limit reached
   - Prevents filesystem inode exhaustion

**Eviction Priority** (lowest to highest):

1. Expired entries (TTL exceeded)
2. Least recently accessed entries
3. Mutable entries over immutable
4. Largest entries (if size-constrained)

#### Cache Operations

**Read Path**:

1. Check memory cache
2. If hit and not expired: return data
3. If miss: check disk cache
4. If hit on disk and not expired: load into memory, return data
5. If miss or expired: fetch from registry, write to both caches

**Write Path**:

1. Write to memory cache (update or insert)
2. Write to disk cache (atomic file write via temp + rename)
3. Update cache metadata (access time, hit stats)
4. If memory cache full: evict LRU entry

**Invalidation**:

1. **Explicit**: User runs `rex cache clear`
2. **Logout**: Clear cache for specific registry
3. **Expiration**: Automatic based on TTL
4. **Manual Refresh**: CLI `--no-cache` flag or TUI refresh action

#### Cache Management

**CLI Commands**:

- `rex registry cache stats` - Show cache size, hit rate, entry count
- `rex registry cache clear` - Clear all cache for registry
- `rex registry cache prune` - Remove expired entries for registry

**Automatic Maintenance**:

- Run cleanup on startup if last cleanup > 24 hours ago
- Background cleanup in TUI mode
- Atomic operations prevent corruption
- Graceful handling of partial/corrupted files

#### Cache Key Generation

**Registry Hash**:

- Hash registry URL to create safe directory name
- Use SHA256 truncated to 16 hex chars
- Collision-resistant, deterministic
- Handles special characters in URLs

**Repository Hash**:

- Hash repository name to create subdirectory
- Prevents issues with special characters (/, :, etc.)
- Avoids filesystem path length limits

**Example**:

- Registry: `localhost:5000` → hash: `a3f5c8e9b2d1f4a7`
- Repository: `my/app` → hash: `d4b6c7f1a8e3d2c5`
- Path: `~/.cache/rex/tags/a3f5c8e9b2d1f4a7/d4b6c7f1a8e3d2c5/tags.json`

#### Configuration

**Cache Settings** (in config.toml):

```toml
[cache]
enabled = true
location = "~/.cache/rex"

[cache.ttl]
catalog = 300        # 5 minutes
tags = 300           # 5 minutes
manifest = 86400     # 24 hours
config = 86400       # 24 hours

[cache.limits]
memory_entries = 1000
memory_size_mb = 100
disk_entries = 10000
disk_size_mb = 1024
max_age_days = 30

[cache.behavior]
consistency = "weak"  # weak, strong, eventual
serve_stale = true    # Serve expired data while refreshing
```

#### Error Handling

**Cache Failures**:

- Cache read errors: Log warning, fetch from registry
- Cache write errors: Log warning, continue without caching
- Disk full: Evict entries, log error if eviction fails
- Corrupted cache files: Delete corrupted file, re-fetch from registry

**Graceful Degradation**:

- Cache is optional enhancement, not required for operation
- All operations work without cache (just slower)
- Corrupted cache doesn't break CLI functionality
- Missing cache directory created automatically

#### Performance Considerations

**Read Performance**:

- Memory cache hit: <1μs (hash map lookup)
- Disk cache hit: 1-10ms (file I/O)
- Registry fetch: 50-500ms (network latency)
- Goal: >80% cache hit rate for repeated operations

**Write Performance**:

- Async writes to disk (non-blocking)
- Batch metadata updates when possible
- Use temp files to prevent partial writes
- Minimal overhead for CLI operations

**Concurrency**:

- Support concurrent CLI invocations safely
- Use file locking for cache writes (flock/fcntl)
- Detect and handle lock timeouts gracefully
- Read operations don't require locks (read-only)

### 1.10 Format Module (`format/`)

Data formatting utilities for human-readable output.

#### Size Formatting

**Purpose**: Convert byte sizes to human-readable format

**Responsibilities**:

- Convert bytes to appropriate units (B, KB, MB, GB, TB)
- Support both decimal (1000-based) and binary (1024-based) units
- Configurable precision
- Handle zero and very large numbers

**Examples**:

- `1024` → "1.0 KB" (binary) or "1.0 kB" (decimal)
- `1048576` → "1.0 MB"
- `5368709120` → "5.0 GB"

#### Timestamp Formatting

**Purpose**: Format ISO 8601 timestamps for display

**Responsibilities**:

- Parse ISO 8601 format (RFC 3339)
- Format as absolute time: "2024-01-15 14:30:00 UTC"
- Format as relative time: "2 hours ago", "3 days ago", "just now"
- Handle timezone conversions
- Deal with missing or invalid timestamps

**Relative Time Examples**:

- < 1 minute: "just now"
- < 1 hour: "X minutes ago"
- < 1 day: "X hours ago"
- < 1 week: "X days ago"
- < 1 month: "X weeks ago"
- >= 1 month: show absolute date

### 1.11 Error Module (`error/`)

Comprehensive error handling with user-friendly messages.

#### Error Categories

**Network Errors**:

- Connection refused
- Timeout
- DNS resolution failure
- SSL/TLS errors

**Authentication Errors**:

- Invalid credentials (401)
- Insufficient permissions (403)
- Token expired
- Missing authentication

**Resource Errors**:

- Repository not found (404)
- Tag not found (404)
- Manifest not found (404)

**Rate Limiting**:

- Too many requests (429)
- Retry-After header handling

**Server Errors**:

- Internal server error (500)
- Service unavailable (503)
- Registry-specific errors

**Validation Errors**:

- Invalid manifest format
- Invalid image reference
- Digest mismatch
- Invalid URL

**Configuration Errors**:

- Invalid config file
- Missing required settings
- Invalid credentials format

#### Error Context

**Include**:

- Root cause (underlying error)
- Contextual information (which operation failed)
- Suggested remediation (what user should try)
- Relevant HTTP status codes
- Registry error messages (from response body)

**Example Error Messages**:

- "Failed to connect to registry at https://localhost:5000: connection refused.
  Is the registry running?"
- "Authentication failed for localhost:5000: invalid username or password.
  Use 'rex login' to update credentials."
- "Repository 'myrepo' not found in registry localhost:5000.
  Use 'rex repos' to list available repositories."

### 1.12 Logging Strategy

Rex uses structured logging throughout the application for debugging, diagnostics, and observability.

#### Logging Framework

**Library**: Use `tracing` crate (Rust standard for async-aware structured logging)

**Why `tracing` over `log`**:

- Structured logging with fields (key-value pairs)
- Async-aware (important for async HTTP operations)
- Span-based tracing for request context
- Better performance and flexibility
- Rich ecosystem (tracing-subscriber, tracing-appender)

#### Log Levels

**TRACE**:

- Very detailed diagnostic information
- Function entry/exit
- Loop iterations
- Cache lookups

**DEBUG**:

- Detailed debugging information
- HTTP request/response headers
- Cache hits/misses
- Authentication flow steps
- Parsing intermediate results

**INFO**:

- General informational messages
- Operation start/completion
- Registry connections
- Cache statistics
- Configuration loading

**WARN**:

- Potentially problematic situations
- Cache write failures (fallback to no-cache)
- Using default values due to missing config
- Deprecated features in use
- Retry attempts

**ERROR**:

- Error conditions that prevent operations
- Network failures
- Authentication failures
- Invalid manifests
- File I/O errors

#### Logging Guidelines

**What to Log**:

1. **Registry Operations**:
   - `INFO`: "Connecting to registry: {url}"
   - `INFO`: "Fetching catalog from {registry}"
   - `DEBUG`: "GET {url} - Status: {status}, Duration: {duration}ms"
   - `ERROR`: "Failed to connect to {registry}: {error}"

2. **Authentication**:
   - `INFO`: "Authenticating with {registry} using {method}"
   - `DEBUG`: "Token acquired, expires in {ttl}s"
   - `WARN`: "Token expired, refreshing"
   - `ERROR`: "Authentication failed: {reason}"
   - **NEVER**: Log passwords, tokens, or sensitive credentials

3. **Cache Operations**:
   - `INFO`: "Loading cache from {path}"
   - `DEBUG`: "Cache hit: {key}"
   - `DEBUG`: "Cache miss: {key}, fetching from registry"
   - `DEBUG`: "Cache write: {key}, size: {size} bytes"
   - `WARN`: "Cache write failed: {error}, continuing without cache"

4. **HTTP Requests**:
   - `DEBUG`: "Request: {method} {url}"
   - `DEBUG`: "Response: {status} in {duration}ms"
   - `TRACE`: "Request headers: {headers}"
   - `TRACE`: "Response headers: {headers}"
   - `DEBUG`: "Retrying request ({attempt}/{max_attempts}): {reason}"

5. **Configuration**:
   - `INFO`: "Loading configuration from {path}"
   - `DEBUG`: "Config: {key} = {value}"
   - `WARN`: "Config file not found, using defaults"
   - `ERROR`: "Invalid config: {error}"

**What NOT to Log**:

- Passwords (in plaintext or encoded)
- Authentication tokens
- Full blob/layer content
- User's personal information
- Full file paths containing usernames (use relative paths)

#### Structured Fields

Use structured fields for consistent querying and filtering:

```rust
// Good - structured fields
tracing::info!(
    registry = %url,
    repository = %repo,
    duration_ms = duration.as_millis(),
    "Fetched repository tags"
);

// Bad - unstructured string
tracing::info!("Fetched tags for {} from {} in {}ms", repo, url, duration);
```

**Common Field Names**:

- `registry` - Registry URL
- `repository` - Repository name
- `tag` - Tag name
- `digest` - Content digest
- `duration_ms` - Operation duration in milliseconds
- `status` - HTTP status code
- `error` - Error message
- `attempt` - Retry attempt number
- `cache_key` - Cache key

#### Spans for Context

Use spans to add context to groups of operations:

```rust
// Create span for entire registry operation
let span = tracing::info_span!("fetch_manifest",
    registry = %registry,
    repository = %repo,
    reference = %tag
);

let _enter = span.enter();
// All logs within this scope automatically include span fields
```

**Span Usage**:

- HTTP requests (track request lifecycle)
- Registry operations (catalog, tags, manifest fetch)
- Authentication flow
- Cache operations
- Image inspection (multiple sub-operations)

#### Log Output Configuration

**CLI Mode**:

- Default: Show only WARN and ERROR to stderr
- `--verbose` / `-v`: Show INFO and above
- `-vv`: Show DEBUG and above
- `-vvv`: Show TRACE (everything)
- Use human-readable format with timestamps

**TUI Mode**:

- Log to file by default: `~/.local/state/rex/rex.log`
- Don't pollute the UI with log output
- User can view logs in a debug panel (optional)

**Log Format**:

**Human-readable** (CLI, TTY):

```text
2024-01-15 14:30:45 UTC INFO  Connecting to registry: localhost:5000
2024-01-15 14:30:45 UTC DEBUG GET /v2/_catalog - Status: 200, Duration: 45ms
```

**JSON** (when piped or for log aggregation):

```json
{"timestamp":"2024-01-15T14:30:45Z","level":"INFO","message":"Connecting to registry","registry":"localhost:5000"}
```

#### Log File Management

**Location**: `~/.local/state/rex/rex.log` (XDG State Directory)

**Rotation**:

- Rotate when file exceeds 10MB
- Keep last 5 log files
- Use dated filenames: `rex.log`, `rex.log.1`, `rex.log.2`, etc.

**Cleanup**:

- Delete logs older than 7 days
- User can configure retention in config file

#### Environment Variables

**RUST_LOG**: Standard Rust log filtering

```bash
# Show all rex logs at debug level
RUST_LOG=rex=debug rex repos

# Show HTTP client at trace level
RUST_LOG=rex::client=trace rex repos

# Show everything at trace level
RUST_LOG=trace rex repos
```

**REX_LOG_FORMAT**: Override log format

```bash
# Force JSON output
REX_LOG_FORMAT=json rex repos

# Force human-readable
REX_LOG_FORMAT=pretty rex repos
```

#### Integration with Error Handling

Errors should be both logged and returned:

```rust
// Log error with context, then return
if let Err(e) = fetch_manifest() {
    tracing::error!(
        registry = %registry,
        repository = %repo,
        error = %e,
        "Failed to fetch manifest"
    );
    return Err(e);
}
```

**Error Context**:

- Errors bubble up with context
- Each layer can add contextual information
- Top-level logs the complete error chain
- Avoid logging the same error multiple times

#### Performance Considerations

**Minimal Overhead**:

- Logs at disabled levels have near-zero cost
- Structured fields use lazy evaluation
- Spans use RAII for automatic cleanup

**Async-Aware**:

- Spans automatically propagate through async boundaries
- Works correctly with tokio runtime

**Sampling** (future):

- For high-frequency operations, sample logs
- Example: Log 1% of cache hits, all cache misses

### 1.13 Public API Design

The core library exposes a high-level API that hides complexity from consumers.

#### Primary Entry Point

**Rex Struct**:

- Main facade for all registry operations
- Encapsulates client, auth, and config
- Provides simple, intuitive methods
- Handles common workflows

**Creation**:

- Connect to registry with auto-detection
- Connect with explicit configuration
- Connect with custom authentication

**Operations**:

- List repositories (handles pagination automatically)
- List tags for repository (handles pagination)
- Search repositories (with fuzzy matching)
- Search tags (with fuzzy matching)
- Inspect image (fetches manifest + config in one call)
- Get manifest (with content negotiation)
- Get image config
- Resolve tag to digest

#### High-Level Data Structures

**ImageInfo**:

- Combines manifest, config, and derived information
- Provides complete view of an image
- Calculates total size across all layers
- Lists all platforms (for multi-arch images)
- Ready for display without additional processing

**Design Principle**: Library users should not need to understand OCI spec
details to perform common operations.

---
