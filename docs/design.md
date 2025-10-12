# Rex - Design Document

## Architecture Overview

Rex is architected in three layers:

1. **Core Engine (Library)**: Common functionality for registry operations,
   authentication, and data management
2. **CLI Interface**: Non-interactive command-line interface built on top of
   the core engine
3. **TUI Interface**: Interactive terminal UI built on top of the core engine

This layered architecture ensures code reuse, consistent behavior across
interfaces, and clean separation of concerns.

```text
┌─────────────────────────────────────────┐
│         CLI Interface (rex)             │
│  - Argument parsing                     │
│  - Output formatting                    │
│  - TTY detection                        │
└─────────────────────────────────────────┘
              │
              │  uses
              ▼
┌─────────────────────────────────────────┐
│       Core Engine (librex)              │
│  - Registry client                      │
│  - Authentication                       │
│  - OCI operations                       │
│  - Data models                          │
└─────────────────────────────────────────┘
              ▲
              │  uses
              │
┌─────────────────────────────────────────┐
│         TUI Interface (rex tui)         │
│  - Event handling                       │
│  - UI components                        │
│  - Navigation                           │
└─────────────────────────────────────────┘
```

---

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

## Part 2: CLI Interface Design

The CLI provides a non-interactive command-line interface for scripting and
automation. It is built on top of the core engine and handles argument parsing,
output formatting, and terminal detection.

### 2.1 Root Command

The root command provides access to all registry operations through
subcommands. When invoked without a subcommand, it displays help information.

**Command**: `rex [OPTIONS] <SUBCOMMAND>`

**Description**: Explore OCI/Docker container registries

**Behavior**:

- Without subcommand: Display help message and exit
- With `--version`: Display version information and exit
- With `--help`: Display detailed help and exit
- With subcommand: Execute the specified subcommand

### 2.2 Global Options

Global options can be specified before any subcommand and affect all
operations.

#### Registry Selection

**`--registry, -r <URL>`**

- Override the current/default registry for this command
- Format: `[http://|https://]host[:port]` or registry name
- Takes precedence over current registry
- Examples:
  - `rex -r localhost:8080 repos`
  - `rex --registry https://ghcr.io repos`
  - `rex -r myregistry repos` (use named registry)

**Default Registry Resolution**:

1. `--registry` flag if provided (highest priority)
2. Current registry (set via `rex registry use`)
3. `localhost:5000` (Zot default)

#### Output Format

**`--format, -f <FORMAT>`**

- Control output format
- Values: `pretty`, `json`, `yaml`
- Default: `pretty` when stdout is TTY, `yaml` when piped
- Examples:
  - `rex -f json repos`
  - `rex --format yaml tags myrepo`

**`--quiet, -q`**

- Suppress visual feedback (progress bars, spinners, status messages)
- Show only essential output data
- Useful for minimal output in scripts
- Does not change output format (use `--format` for that)
- Example: `rex -q repos`

#### Color Control

**`--color <WHEN>`**

- Control colored output
- Values: `auto`, `always`, `never`
- Default: `auto` (colors when stdout is TTY)
- Examples:
  - `rex --color always repos`
  - `rex --color never repos`

**`--no-color`**

- Disable colored output
- Alias for `--color never`
- Example: `rex --no-color repos`

#### Verbosity

**`--verbose, -v`**

- Increase logging verbosity
- Can be repeated for more verbosity:
  - `-v`: INFO level and above
  - `-vv`: DEBUG level and above
  - `-vvv`: TRACE level (everything)
- Default: WARN and ERROR only
- Example: `rex -vv repos`

#### Network Options

**`--timeout <SECONDS>`**

- Set request timeout in seconds
- Default: 30 seconds
- Example: `rex --timeout 60 repos`

**`--no-verify-tls`**

- Skip TLS certificate verification
- Useful for self-signed certificates
- Security warning displayed when used
- Example: `rex --no-verify-tls repos`

#### Cache Control

**`--no-cache`**

- Bypass cache and fetch fresh data from registry
- Forces strong consistency
- Useful for ensuring up-to-date information
- Example: `rex --no-cache repos`

**`--cache-dir <PATH>`**

- Override default cache directory
- Default: `~/.cache/rex/`
- Example: `rex --cache-dir /tmp/rex-cache repos`

#### Configuration

**`--config <PATH>`**

- Use custom configuration file
- Default: `~/.config/rex/config.toml`
- Example: `rex --config ./my-config.toml repos`

#### Help and Version

**`--help, -h`**

- Display help information
- Shows subcommands and global options
- Example: `rex --help`

**`--version, -V`**

- Display version information
- Shows rex version and build information
- Example: `rex --version`

### 2.3 Configuration Commands

Manage rex configuration settings.

#### `rex config`

Display all configuration settings.

**Command**: `rex config [OPTIONS]`

**Options**:

- `--format, -f <FORMAT>`: Output format (`pretty`, `json`, `yaml`)

**Behavior**:

- Displays all configuration values from `~/.config/rex/config.toml`
- Shows current values with indication of which are defaults vs explicitly set
- Can output in different formats for scripting

**Examples**:

```bash
# Display all configuration
rex config

# Display in JSON format
rex config --format json
```

**Output** (pretty format):

```text
Output:
  format = pretty (default)
  color = auto (default)
  quiet = false (default)

Network:
  timeout = 30 (default)
  retry_attempts = 3 (default)
  verify_tls = true (default)

Cache:
  enabled = true (default)
  ttl.catalog = 300 (default)
  ttl.tags = 300 (default)
  ttl.manifest = 86400 (default)
  behavior.consistency = weak (default)

TUI:
  theme = dark (default)
  vim_bindings = true (default)
```

#### `rex config get`

Get a specific configuration value.

**Command**: `rex config get <KEY>`

**Arguments**:

- `<KEY>`: Configuration key in dotted notation (e.g., `output.format`)

**Behavior**:

- Returns the value of the specified configuration key
- Returns default value if not explicitly set
- Exits with error if key is invalid

**Examples**:

```bash
# Get output format
rex config get output.format

# Get network timeout
rex config get network.timeout

# Get cache TTL for catalogs
rex config get cache.ttl.catalog
```

**Output**:

```text
pretty
```

#### `rex config set`

Set a configuration value.

**Command**: `rex config set <KEY> <VALUE>`

**Arguments**:

- `<KEY>`: Configuration key in dotted notation
- `<VALUE>`: Value to set (type depends on key)

**Behavior**:

- Updates the specified configuration key in config file
- Creates section if it doesn't exist
- Validates value based on key type
- Takes effect immediately for subsequent commands

**Configuration Keys**:

**Output Section** (`output.*`):

- `output.format`: Default output format (`pretty`, `json`, `yaml`)
- `output.color`: Color output behavior (`auto`, `always`, `never`)
- `output.quiet`: Suppress visual feedback (`true`, `false`)

**Network Section** (`network.*`):

- `network.timeout`: Request timeout in seconds (integer)
- `network.retry_attempts`: Number of retry attempts (integer)
- `network.verify_tls`: Verify TLS certificates (`true`, `false`)

**Cache Section** (`cache.*`):

- `cache.enabled`: Enable caching (`true`, `false`)
- `cache.ttl.catalog`: Catalog cache TTL in seconds (integer)
- `cache.ttl.tags`: Tags cache TTL in seconds (integer)
- `cache.ttl.manifest`: Manifest cache TTL in seconds (integer)
- `cache.ttl.config`: Config cache TTL in seconds (integer)
- `cache.limits.memory_entries`: Max memory cache entries (integer)
- `cache.limits.memory_size_mb`: Max memory cache size in MB (integer)
- `cache.limits.disk_entries`: Max disk cache entries (integer)
- `cache.limits.disk_size_mb`: Max disk cache size in MB (integer)
- `cache.behavior.consistency`: Cache consistency level
  (`weak`, `strong`, `eventual`)
- `cache.behavior.serve_stale`: Serve stale data while refreshing
  (`true`, `false`)

**TUI Section** (`tui.*`):

- `tui.theme`: Color theme (`dark`, `light`)
- `tui.vim_bindings`: Enable vim keybindings (`true`, `false`)

**Examples**:

```bash
# Set default output format to JSON
rex config set output.format json

# Enable colored output always
rex config set output.color always

# Set network timeout to 60 seconds
rex config set network.timeout 60

# Disable TLS verification
rex config set network.verify_tls false

# Set cache TTL for catalogs to 10 minutes
rex config set cache.ttl.catalog 600

# Enable vim bindings for TUI
rex config set tui.vim_bindings true
```

**Output**:

```text
✓ Set output.format = json
```

#### `rex config edit`

Open configuration file in system editor.

**Command**: `rex config edit`

**Behavior**:

- Opens `~/.config/rex/config.toml` in the system editor
- Uses `$EDITOR` environment variable if set
- Falls back to common editors: `vim`, `nano`, `vi` (in that order)
- Creates config file if it doesn't exist
- Validates configuration after editing

**Examples**:

```bash
# Edit configuration in default editor
rex config edit

# Using specific editor
EDITOR=code rex config edit
```

**Configuration File Format**:

```toml
[output]
format = "pretty"
color = "auto"
quiet = false

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

[cache.limits]
memory_entries = 1000
memory_size_mb = 100
disk_entries = 10000
disk_size_mb = 1024

[cache.behavior]
consistency = "weak"
serve_stale = true

[tui]
theme = "dark"
vim_bindings = true
```

### 2.4 Registry Management Commands

Commands for managing registry configurations.

#### `rex registry init`

Add and configure a new registry.

**Command**: `rex registry init <NAME> <URL> [OPTIONS]`

**Arguments**:

- `<NAME>`: Unique name for this registry (e.g., `local`, `prod`, `docker`)
- `<URL>`: Registry URL in format `[http://|https://]host[:port]`

**Options**:

- `--set-current`: Set this registry as the current/default after adding
- `--insecure`: Allow insecure HTTP connections (default: require HTTPS)
- `--skip-verify`: Skip TLS certificate verification

**Behavior**:

- Adds registry configuration to `~/.config/rex/config.toml`
- Validates registry by connecting and checking `/v2/` endpoint
- Optionally sets as current registry
- Prompts for authentication if registry requires it

**Examples**:

```bash
# Add local Zot registry and set as current
rex registry init local http://localhost:5000 --set-current

# Add production registry
rex registry init prod https://registry.example.com

# Add registry with insecure connection
rex registry init dev http://dev.registry.local --insecure --set-current

# Add Docker Hub
rex registry init dockerhub https://registry-1.docker.io
```

**Output** (pretty format):

```text
✓ Connected to http://localhost:5000
✓ Registry 'local' added successfully
✓ Set 'local' as current registry
```

#### `rex registry use`

Select a configured registry as the current/default.

**Command**: `rex registry use <NAME>`

**Arguments**:

- `<NAME>`: Name of the registry to use (from configured registries)

**Behavior**:

- Sets the specified registry as current in config file
- All subsequent commands use this registry unless overridden with `--registry`
- Validates registry is still accessible

**Examples**:

```bash
# Switch to production registry
rex registry use prod

# Switch to local registry
rex registry use local
```

**Output** (pretty format):

```text
✓ Current registry set to 'prod' (https://registry.example.com)
```

#### `rex registry list`

List all configured registries.

**Command**: `rex registry list [OPTIONS]`

**Aliases**: `rex registry ls`

**Options**:

- Standard output options: `--format`, `--quiet`

**Behavior**:

- Lists all registries from config file
- Shows which registry is current
- Displays URL and connection status

**Examples**:

```bash
# List all registries
rex registry list

# List in JSON format
rex registry ls --format json
```

**Output** (pretty format):

```text
CURRENT  NAME        URL                            STATUS
*        local       http://localhost:5000          online
         prod        https://registry.example.com   online
         dockerhub   https://registry-1.docker.io   online
```

**Output** (JSON format):

```json
{
  "current": "local",
  "registries": [
    {
      "name": "local",
      "url": "http://localhost:5000",
      "current": true,
      "status": "online"
    },
    {
      "name": "prod",
      "url": "https://registry.example.com",
      "current": false,
      "status": "online"
    }
  ]
}
```

#### `rex registry login`

Authenticate with a registry.

**Command**: `rex registry login [NAME] [OPTIONS]`

**Arguments**:

- `[NAME]`: Registry name (optional, uses current registry if omitted)

**Options**:

- `--username, -u <USERNAME>`: Username for authentication
- `--password, -p <PASSWORD>`: Password (not recommended, use prompt)
- `--password-stdin`: Read password from stdin

**Behavior**:

- Authenticates with the specified registry
- Prompts for username/password if not provided
- Stores credentials securely in OS keychain
- Falls back to Docker/Podman auth files if present
- Tests authentication by making a request to `/v2/`

**Examples**:

```bash
# Login to current registry (interactive prompt)
rex registry login

# Login to named registry
rex registry login prod

# Login with username (prompts for password)
rex registry login local --username myuser

# Login with credentials from stdin
echo "$PASSWORD" | rex registry login prod --username myuser --password-stdin
```

**Output**:

```text
Username: myuser
Password:
✓ Successfully authenticated with localhost:5000
✓ Credentials stored securely
```

#### `rex registry logout`

Remove authentication for a registry.

**Command**: `rex registry logout [NAME]`

**Arguments**:

- `[NAME]`: Registry name (optional, uses current registry if omitted)

**Behavior**:

- Removes stored credentials for the registry
- Clears credentials from OS keychain
- Does not remove registry from configuration

**Examples**:

```bash
# Logout from current registry
rex registry logout

# Logout from named registry
rex registry logout prod
```

**Output**:

```text
✓ Logged out from localhost:5000
✓ Credentials removed
```

#### `rex registry remove`

Remove a registry from configuration.

**Command**: `rex registry remove <NAME>`

**Aliases**: `rex registry rm`

**Arguments**:

- `<NAME>`: Registry name to remove

**Options**:

- `--force, -f`: Skip confirmation prompt

**Behavior**:

- Removes registry from configuration
- Also removes stored credentials
- Prevents removal of current registry without confirmation
- If current registry is removed, resets to default (localhost:5000)

**Examples**:

```bash
# Remove a registry (with confirmation)
rex registry remove old-registry

# Force remove without confirmation
rex registry rm dev --force
```

**Output**:

```text
Remove registry 'old-registry' (https://old.example.com)? [y/N]: y
✓ Registry 'old-registry' removed
✓ Credentials removed
```

#### `rex registry cache stats`

Show cache statistics for a registry.

**Command**: `rex registry cache stats [NAME] [OPTIONS]`

**Arguments**:

- `[NAME]`: Registry name (optional, uses current registry if omitted)

**Options**:

- `--format, -f <FORMAT>`: Output format (`pretty`, `json`, `yaml`)

**Behavior**:

- Shows cache statistics for the specified registry
- Displays hit rate, entry count, total size
- Shows cache breakdown by type (catalog, tags, manifests, configs)
- Shows last cleanup time

**Examples**:

```bash
# Show cache stats for current registry
rex registry cache stats

# Show stats for specific registry
rex registry cache stats prod

# JSON output
rex registry cache stats --format json
```

**Output** (pretty format):

```text
Cache Statistics for 'local' (http://localhost:5000)

Overview:
  Total Entries: 245
  Total Size: 12.3 MB
  Memory Cache: 89 entries (2.1 MB)
  Disk Cache: 245 entries (12.3 MB)
  Hit Rate: 87.4%
  Last Cleanup: 2 hours ago

By Type:
  Catalogs:   5 entries, 45 KB
  Tags:       28 entries, 156 KB
  Manifests:  142 entries, 8.2 MB
  Configs:    70 entries, 3.9 MB

Cache Location: ~/.cache/rex/a3f5c8e9b2d1f4a7/
```

**Output** (JSON format):

```json
{
  "registry": "local",
  "url": "http://localhost:5000",
  "total_entries": 245,
  "total_size": 12898304,
  "memory_entries": 89,
  "memory_size": 2201600,
  "disk_entries": 245,
  "disk_size": 12898304,
  "hit_rate": 0.874,
  "last_cleanup": "2024-01-15T08:30:00Z",
  "by_type": {
    "catalogs": {
      "entries": 5,
      "size": 46080
    },
    "tags": {
      "entries": 28,
      "size": 159744
    },
    "manifests": {
      "entries": 142,
      "size": 8601600
    },
    "configs": {
      "entries": 70,
      "size": 4090880
    }
  },
  "cache_path": "~/.cache/rex/a3f5c8e9b2d1f4a7/"
}
```

#### `rex registry cache clear`

Clear cache for a registry.

**Command**: `rex registry cache clear [NAME] [OPTIONS]`

**Arguments**:

- `[NAME]`: Registry name (optional, uses current registry if omitted)

**Options**:

- `--all`: Clear cache for all registries
- `--type <TYPE>`: Clear only specific cache type
  (`catalog`, `tags`, `manifest`, `config`)
- `--force, -f`: Skip confirmation prompt

**Behavior**:

- Removes all cached data for the specified registry
- Can target specific cache types
- Prompts for confirmation unless `--force` is used
- Clears both memory and disk cache

**Examples**:

```bash
# Clear cache for current registry (with confirmation)
rex registry cache clear

# Clear cache for specific registry
rex registry cache clear prod

# Clear all registry caches
rex registry cache clear --all

# Clear only manifest cache
rex registry cache clear --type manifest

# Force clear without confirmation
rex registry cache clear --force
```

**Output**:

```text
Clear cache for 'local' (http://localhost:5000)? [y/N]: y
✓ Cleared 245 entries (12.3 MB)
✓ Cache cleared successfully
```

#### `rex registry cache prune`

Remove expired cache entries for a registry.

**Command**: `rex registry cache prune [NAME] [OPTIONS]`

**Arguments**:

- `[NAME]`: Registry name (optional, uses current registry if omitted)

**Options**:

- `--all`: Prune cache for all registries
- `--dry-run`: Show what would be removed without actually removing

**Behavior**:

- Removes expired entries based on TTL
- Removes entries exceeding cache limits
- Keeps frequently accessed entries
- Shows how much space was freed

**Examples**:

```bash
# Prune cache for current registry
rex registry cache prune

# Prune all registry caches
rex registry cache prune --all

# Dry run to see what would be removed
rex registry cache prune --dry-run
```

**Output**:

```text
Pruning cache for 'local' (http://localhost:5000)...
✓ Removed 42 expired entries (3.2 MB)
✓ Freed 3.2 MB of disk space
Remaining: 203 entries (9.1 MB)
```

**Output** (dry run):

```text
Would remove 42 expired entries (3.2 MB):
  - 8 catalog entries
  - 15 tag entries
  - 12 manifest entries
  - 7 config entries
```

### Configuration File Structure

Registries are stored in `~/.config/rex/config.toml`:

```toml
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
skip_verify = false
```

### 2.5 Image Commands

Commands for exploring images and their details.

#### `rex image`

List all images in the registry.

**Command**: `rex image [OPTIONS]`

**Options**:

- `--format, -f <FORMAT>`: Output format (`pretty`, `json`, `yaml`)
- `--quiet, -q`: Show only image names
- `--filter <PATTERN>`: Filter images by pattern (supports fuzzy matching)
- `--limit <N>`: Limit number of results

**Behavior**:

- Lists all repositories (images) in the current registry
- Shows image name, tag count, and last updated timestamp
- Supports fuzzy search filtering
- Uses cache by default (use `--no-cache` to fetch fresh)

**Examples**:

```bash
# List all images
rex image

# List with filter
rex image --filter alpine

# List in JSON format
rex image --format json

# List only names (quiet mode)
rex image --quiet
```

**Output** (pretty format):

```text
NAME              TAGS    LAST UPDATED
alpine            5       2 hours ago
nginx             12      1 day ago
myapp             3       3 days ago
postgres          8       1 week ago
```

**Output** (JSON format):

```json
{
  "images": [
    {
      "name": "alpine",
      "tags": 5,
      "last_updated": "2024-01-15T10:30:00Z"
    },
    {
      "name": "nginx",
      "tags": 12,
      "last_updated": "2024-01-14T14:20:00Z"
    }
  ]
}
```

#### `rex image <NAME>`

List all tags for a specific image.

**Command**: `rex image <NAME> [OPTIONS]`

**Arguments**:

- `<NAME>`: Image name (repository)

**Options**:

- `--format, -f <FORMAT>`: Output format (`pretty`, `json`, `yaml`)
- `--quiet, -q`: Show only tag names
- `--filter <PATTERN>`: Filter tags by pattern
- `--sort <FIELD>`: Sort by field (`name`, `date`, `size`)
- `--limit <N>`: Limit number of results

**Behavior**:

- Lists all tags for the specified image
- Shows details for each tag (digest, size, created date, platforms)
- Supports sorting and filtering
- Shows multi-platform indicator

**Examples**:

```bash
# List all tags for alpine
rex image alpine

# List with sorting by date (newest first)
rex image alpine --sort date

# Filter tags
rex image myapp --filter v1

# Show only tag names
rex image alpine --quiet
```

**Output** (pretty format):

```text
TAG       DIGEST                                                      SIZE     CREATED      PLATFORMS
latest    sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672...  7.3 MB   2 hours ago  linux/amd64, linux/arm64
3.19      sha256:13b7e62e8df80264dbb747995705a986aa530415763a6c5...  7.3 MB   2 hours ago  linux/amd64, linux/arm64, linux/arm/v7
3.18      sha256:a606584aa9aa875552092ec49dd9db890d897848f8ec0a6...  7.2 MB   1 month ago  linux/amd64, linux/arm64
edge      sha256:beefdbd8a1da6d2915566fde36db9db0b524eb737fc57c...  7.4 MB   1 day ago    linux/amd64, linux/arm64
```

**Output** (JSON format):

```json
{
  "name": "alpine",
  "tags": [
    {
      "tag": "latest",
      "digest": "sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b",
      "size": 7651328,
      "created": "2024-01-15T10:30:00Z",
      "platforms": ["linux/amd64", "linux/arm64"]
    }
  ]
}
```

#### `rex image <NAME>:<TAG>`

Show details for a specific image tag.

**Command**: `rex image <NAME>:<TAG> [OPTIONS]`

**Arguments**:

- `<NAME>:<TAG>`: Image reference (can also use `@digest`)

**Options**:

- `--format, -f <FORMAT>`: Output format (`pretty`, `json`, `yaml`)
- `--platform <OS/ARCH>`: Show details for specific platform (multi-arch)

**Behavior**:

- Shows summary information about the image
- Includes manifest type, config details, layer count
- For multi-arch images, shows all platforms or specific platform
- Does not show full details (use `inspect` for that)

**Examples**:

```bash
# Show details for alpine:latest
rex image alpine:latest

# Show for specific platform
rex image alpine:latest --platform linux/arm64

# Show using digest
rex image alpine@sha256:c5b1261d...

# JSON output
rex image alpine:latest --format json
```

**Output** (pretty format):

```text
Image: alpine:latest
Digest: sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b
Type: OCI Image Index (multi-platform)
Size: 7.3 MB

Platforms:
  linux/amd64
  linux/arm64
  linux/arm/v7

Configuration:
  Entrypoint: /bin/sh
  Env: PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
  WorkingDir: /
  User: (empty)

Layers: 1
  sha256:4abcf2066143... (7.3 MB)

Labels:
  maintainer: Natanael Copa <ncopa@alpinelinux.org>
  version: 3.19.0

Created: 2024-01-15T10:30:00Z
```

**Output** (JSON format):

```json
{
  "reference": "alpine:latest",
  "digest": "sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b",
  "manifest_type": "OCI Image Index",
  "size": 7651328,
  "platforms": ["linux/amd64", "linux/arm64", "linux/arm/v7"],
  "config": {
    "entrypoint": ["/bin/sh"],
    "env": ["PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"],
    "working_dir": "/",
    "user": ""
  },
  "layers": [
    {
      "digest": "sha256:4abcf2066143...",
      "size": 7651328
    }
  ],
  "labels": {
    "maintainer": "Natanael Copa <ncopa@alpinelinux.org>",
    "version": "3.19.0"
  },
  "created": "2024-01-15T10:30:00Z"
}
```

#### `rex image inspect`

Show complete detailed information about an image.

**Command**: `rex image inspect <NAME>:<TAG> [OPTIONS]`

**Arguments**:

- `<NAME>:<TAG>`: Image reference (can also use `@digest`)

**Options**:

- `--format, -f <FORMAT>`: Output format (`pretty`, `json`, `yaml`)
- `--platform <OS/ARCH>`: Inspect specific platform (multi-arch images)
- `--raw-manifest`: Show raw manifest JSON
- `--raw-config`: Show raw config JSON

**Behavior**:

- Shows complete information about the image
- Includes full manifest, complete config, all layers with details
- All metadata, annotations, and labels
- History of layer creation
- Full platform information

**Examples**:

```bash
# Full inspection of alpine:latest
rex image inspect alpine:latest

# Inspect specific platform
rex image inspect alpine:latest --platform linux/arm64

# Show raw manifest
rex image inspect alpine:latest --raw-manifest

# JSON output with all details
rex image inspect alpine:latest --format json
```

**Output** (pretty format, truncated):

```text
Image: alpine:latest
Digest: sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b
Registry: localhost:5000
Type: OCI Image Index (multi-platform)
Media Type: application/vnd.oci.image.index.v1+json
Total Size: 7.3 MB

Manifest Digest: sha256:c5b1261d...
Config Digest: sha256:9c6f0724...

Platforms Available:
  1. linux/amd64
     Digest: sha256:a1b2c3d4...
     Size: 7.3 MB

  2. linux/arm64
     Digest: sha256:e5f6g7h8...
     Size: 7.3 MB

Configuration (linux/amd64):
  Architecture: amd64
  OS: linux
  Created: 2024-01-15T10:30:00Z

  Config:
    User: (empty)
    Env:
      - PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
    Entrypoint:
      - /bin/sh
    WorkingDir: /

  Labels:
    maintainer: Natanael Copa <ncopa@alpinelinux.org>
    version: 3.19.0

Layers (1):
  1. sha256:4abcf20661432fb2d719b4568d94db3b6cf9b44bf2a3e1c2c6d0c89fd9e6e0b2
     Size: 7.3 MB (7,651,328 bytes)
     Media Type: application/vnd.oci.image.layer.v1.tar+gzip

History (1 entries):
  1. Created: 2024-01-15T10:30:00Z
     Size: 7.3 MB

RootFS:
  Type: layers
  DiffIDs:
    - sha256:8e012198eea15b2554b07014081c85fec4967a1b9cc4b65bd9a4bce3ae1c0c88
```

**Output with `--raw-manifest`**:

Shows the complete JSON manifest as returned by the registry.

**Output with `--raw-config`**:

Shows the complete JSON config blob.

### 2.6 Search Commands

Search for images and tags across the registry using fuzzy matching.

#### `rex search image`

Search for images (repositories) by name with optional tag filtering.

**Command**: `rex search image <QUERY> [OPTIONS]`

**Arguments**:

- `<QUERY>`: Image name search query (supports fuzzy matching)

**Options**:

- `--tag <TAG>`: Filter results to only images that have this exact tag
- `--format, -f <FORMAT>`: Output format (`pretty`, `json`, `yaml`)
- `--quiet, -q`: Show only image names
- `--limit <N>`: Limit number of results (default: 50)
- `--exact`: Use exact matching instead of fuzzy matching

**Behavior**:

- Fuzzy searches repository names matching the query
- Without `--tag`: Returns all matching repositories
- With `--tag`: Returns only repositories that have the specified tag
- Results are ordered from best match to least relevant
- Only shows matches with >= 50% accuracy score
- Scores are calculated internally but not displayed

**Examples**:

```bash
# Search for repositories matching "alpine"
rex search image alpine

# Search for repositories matching "alpine" that have tag "3.19"
rex search image alpine --tag 3.19

# Exact match for image name
rex search image alpine --exact

# Limit results
rex search image app --limit 10

# JSON output
rex search image nginx --format json
```

**Output** (without tag filter):

```bash
rex search image alpine
```

```text
alpine
alpine-base
myapp-alpine
```

**Output** (with tag filter):

```bash
rex search image alpine --tag 3.19
```

```text
alpine
alpine-base
```

Note: Only images that have a tag exactly matching "3.19" are shown, but the
image name is still fuzzy matched. Both "alpine" and "alpine-base" have a tag
named "3.19".

**Output** (JSON format, without tag filter):

```json
{
  "query": "alpine",
  "search_type": "image",
  "tag_filter": null,
  "total_results": 3,
  "results": [
    {
      "name": "alpine",
      "tags": 5,
      "last_updated": "2024-01-15T10:30:00Z"
    },
    {
      "name": "alpine-base",
      "tags": 2,
      "last_updated": "2024-01-14T08:20:00Z"
    },
    {
      "name": "myapp-alpine",
      "tags": 3,
      "last_updated": "2024-01-13T12:15:00Z"
    }
  ]
}
```

**Output** (JSON format, with tag filter):

```json
{
  "query": "alpine",
  "search_type": "image",
  "tag_filter": "3.19",
  "total_results": 2,
  "results": [
    {
      "name": "alpine",
      "tags": 5,
      "last_updated": "2024-01-15T10:30:00Z",
      "matching_tag": "3.19"
    },
    {
      "name": "alpine-base",
      "tags": 2,
      "last_updated": "2024-01-14T14:20:00Z",
      "matching_tag": "3.19"
    }
  ]
}
```

#### `rex search tags`

Search for tags with optional image scoping.

**Command**: `rex search tags <QUERY> [OPTIONS]`

**Arguments**:

- `<QUERY>`: Tag search query (supports fuzzy matching)

**Options**:

- `--image <NAME>`: Limit search to tags within this specific image
- `--format, -f <FORMAT>`: Output format (`pretty`, `json`, `yaml`)
- `--quiet, -q`: Show only tag names
- `--limit <N>`: Limit number of results (default: 50)
- `--exact`: Use exact matching instead of fuzzy matching

**Behavior**:

- Without `--image`: Fuzzy searches tags across all repositories
- With `--image`: Fuzzy searches tags only within the specified repository
- Results are ordered from best match to least relevant
- Only shows matches with >= 50% accuracy score
- Scores are calculated internally but not displayed

**Examples**:

```bash
# Search for tags matching "1.1" across all images
rex search tags 1.1

# Search for tags matching "v1" within the "myapp" image
rex search tags v1 --image myapp

# Exact tag match
rex search tags latest --exact

# JSON output
rex search tags alpine --format json
```

**Output** (without image scope):

```bash
rex search tags alpine
```

```text
nginx:alpine
nginx:1.24-alpine
myapp:alpine-v1
node:20-alpine3.19
postgres:16-alpine
```

**Output** (with image scope):

```bash
rex search tags v1 --image myapp
```

```text
v1.0.0
v1.0.1
v1.1.0
v1.2.0
alpine-v1
```

**Output** (JSON format, without image scope):

```json
{
  "query": "alpine",
  "search_type": "tags",
  "image_scope": null,
  "total_results": 5,
  "results": [
    {
      "image": "nginx",
      "tag": "alpine",
      "reference": "nginx:alpine",
      "digest": "sha256:abc123...",
      "size": 23654784
    },
    {
      "image": "nginx",
      "tag": "1.24-alpine",
      "reference": "nginx:1.24-alpine",
      "digest": "sha256:def456...",
      "size": 23821056
    },
    {
      "image": "myapp",
      "tag": "alpine-v1",
      "reference": "myapp:alpine-v1",
      "digest": "sha256:ghi789...",
      "size": 45678912
    },
    {
      "image": "node",
      "tag": "20-alpine3.19",
      "reference": "node:20-alpine3.19",
      "digest": "sha256:jkl012...",
      "size": 89123456
    },
    {
      "image": "postgres",
      "tag": "16-alpine",
      "reference": "postgres:16-alpine",
      "digest": "sha256:mno345...",
      "size": 67890123
    }
  ]
}
```

**Output** (JSON format, with image scope):

```json
{
  "query": "v1",
  "search_type": "tags",
  "image_scope": "myapp",
  "total_results": 5,
  "results": [
    {
      "tag": "v1.0.0",
      "reference": "myapp:v1.0.0",
      "digest": "sha256:aaa111...",
      "size": 45678912
    },
    {
      "tag": "v1.0.1",
      "reference": "myapp:v1.0.1",
      "digest": "sha256:bbb222...",
      "size": 45679123
    },
    {
      "tag": "v1.1.0",
      "reference": "myapp:v1.1.0",
      "digest": "sha256:ccc333...",
      "size": 45780234
    },
    {
      "tag": "v1.2.0",
      "reference": "myapp:v1.2.0",
      "digest": "sha256:ddd444...",
      "size": 45881345
    },
    {
      "tag": "alpine-v1",
      "reference": "myapp:alpine-v1",
      "digest": "sha256:eee555...",
      "size": 46882456
    }
  ]
}
```

**Match Accuracy Threshold**:

- Results are internally scored based on match quality
- Only matches with >= 50% accuracy are displayed
- Results are ordered from best match (highest score) to worst match
  (lowest score above threshold)
- Scoring factors:
  - Exact matches (highest priority)
  - Prefix matches (high priority)
  - Character-by-character fuzzy matches (medium priority)
  - Position of match in string (earlier is better)
  - Length of target string (shorter is better)
- Scores are NOT shown in output, only used for ordering and filtering

**Search Command Summary**:

| Command | Searches | Filter/Scope | Output |
|---------|----------|--------------|--------|
| `rex search image alpine` | Image names | None | Image list |
| `rex search image alpine --tag 3.19` | Image names | Must have tag "3.19" | Filtered image list |
| `rex search tags alpine` | Tags across all images | None | `image:tag` references |
| `rex search tags v1 --image myapp` | Tags within image | Scoped to "myapp" | Tag list |

### 2.7 Version Command

Display version and build information for rex.

#### `rex version`

Show version information.

**Command**: `rex version [OPTIONS]`

**Options**:

- `--verbose, -v`: Show detailed build information
- `--format, -f <FORMAT>`: Output format (`pretty`, `json`, `yaml`)

**Behavior**:

- Displays rex version number
- With `--verbose`: Shows additional build details (commit, build date, etc.)
- Supports multiple output formats for scripting
- Returns exit code 0

**Examples**:

```bash
# Show version
rex version

# Show detailed version information
rex version --verbose

# JSON output
rex version --format json

# YAML output for scripts
rex version --format yaml
```

**Output** (default):

```text
rex 0.1.0
```

**Output** (verbose):

```text
rex 0.1.0
commit: a1b2c3d4
built: 2024-01-15 14:30:00 UTC
rustc: 1.75.0
```

**Output** (JSON format):

```json
{
  "version": "0.1.0",
  "commit": "a1b2c3d4e5f6g7h8",
  "commit_short": "a1b2c3d4",
  "branch": "main",
  "built": "2024-01-15T14:30:00Z",
  "rustc": "1.75.0",
  "target": "x86_64-unknown-linux-gnu",
  "features": ["default", "tls-native"],
  "profile": "release"
}
```

**Output** (YAML format):

```yaml
version: 0.1.0
commit: a1b2c3d4e5f6g7h8
commit_short: a1b2c3d4
branch: main
built: 2024-01-15T14:30:00Z
rustc: 1.75.0
target: x86_64-unknown-linux-gnu
features:
  - default
  - tls-native
profile: release
```

**Version Information Fields**:

- `version`: Semantic version number (X.Y.Z)
  - X: Major version (breaking changes)
  - Y: Minor version (new features, backwards compatible)
  - Z: Patch version (bug fixes)
- `commit`: Full git commit hash (only with `--verbose`)
- `commit_short`: Short git commit hash (8 chars)
- `branch`: Git branch name (only with `--verbose`)
- `built`: Build timestamp in UTC (ISO 8601 format)
- `rustc`: Rust compiler version used to build
- `target`: Target triple (platform, only with `--verbose`)
- `features`: Enabled Cargo features (only with `--verbose`)
- `profile`: Build profile - debug or release (only with `--verbose`)

**Usage in Scripts**:

```bash
# Extract version number
version=$(rex version --format json | jq -r '.version')
echo "Rex version: $version"

# Check version in YAML
rex version --format yaml | grep "^version:" | awk '{print $2}'

# Compare versions
required="0.1.0"
current=$(rex version --format json | jq -r '.version')
if [ "$(printf '%s\n' "$required" "$current" | sort -V | head -n1)" = "$required" ]; then
  echo "Version OK"
else
  echo "Version too old, need $required or higher"
fi

# Get full build info
rex version --verbose --format json | jq '.'
```

**Version Compatibility**:

Rex follows semantic versioning (SemVer):

- **Patch releases** (0.1.0 → 0.1.1): Bug fixes only, fully compatible
- **Minor releases** (0.1.0 → 0.2.0): New features, backwards compatible
- **Major releases** (0.1.0 → 1.0.0): Breaking changes, may require updates

**Build Information Source**:

The version information is embedded at compile time:

- Version from `Cargo.toml`
- Git commit from `git rev-parse HEAD`
- Build timestamp from build time
- Rust version from `rustc --version`
- Target triple from build configuration

### 2.8 Exit Codes

Rex uses standard exit codes to indicate success or failure. This allows scripts
to properly handle errors and make decisions based on the result.

**Exit Code Reference**:

| Code | Name | Description |
|------|------|-------------|
| 0 | Success | Command completed successfully |
| 1 | General Error | Unspecified error occurred |
| 2 | Usage Error | Invalid arguments, missing required parameters, or incorrect syntax |
| 3 | Network Error | Connection failed, timeout, or DNS resolution failure |
| 4 | Authentication Error | Authentication failed (401), insufficient permissions (403) |
| 5 | Not Found | Registry, repository, tag, or manifest not found (404) |
| 6 | Configuration Error | Invalid config file, missing settings, or parse error |
| 7 | Cache Error | Critical cache operation failure |
| 130 | Interrupted | User interrupted the operation (Ctrl+C / SIGINT) |

**Exit Code Details**:

**0 - Success**:

- Command executed successfully
- All requested operations completed
- Data retrieved and displayed (if applicable)

**1 - General Error**:

- Catch-all for errors that don't fit other categories
- Internal errors or unexpected conditions
- Should be rare; most errors use specific codes

**2 - Usage Error**:

- Invalid command syntax
- Missing required arguments
- Unknown flags or options
- Mutually exclusive options used together
- Invalid argument values (e.g., negative timeout)

**3 - Network Error**:

- Connection refused
- Timeout (request or connection)
- DNS resolution failure
- SSL/TLS handshake failure
- Network unreachable
- Broken pipe or connection reset

**4 - Authentication Error**:

- HTTP 401 Unauthorized
- HTTP 403 Forbidden
- Invalid username or password
- Expired or invalid token
- Missing credentials when required

**5 - Not Found**:

- HTTP 404 Not Found
- Registry not found or unreachable
- Repository doesn't exist
- Tag doesn't exist
- Manifest not found

**6 - Configuration Error**:

- Config file not found (when explicitly specified)
- Config file has invalid TOML syntax
- Invalid configuration values
- Required configuration missing
- Config file permissions error

**7 - Cache Error**:

- Critical cache operation failure
- Cache directory not writable
- Cache corruption that prevents operation
- Note: Non-critical cache errors are logged but don't cause exit

**130 - Interrupted**:

- User pressed Ctrl+C (SIGINT)
- Operation was interrupted before completion
- Standard Unix convention for SIGINT

**Usage Examples**:

```bash
# Check if command succeeded
if rex image myapp; then
  echo "Image exists"
else
  echo "Command failed with code: $?"
fi

# Handle specific error types
rex image myrepo
case $? in
  0)
    echo "Success"
    ;;
  4)
    echo "Authentication required. Run: rex registry login"
    ;;
  5)
    echo "Repository not found"
    ;;
  3)
    echo "Network error. Check registry URL and connectivity"
    ;;
  *)
    echo "Unknown error occurred"
    ;;
esac

# Exit on authentication errors but continue on not found
rex image alpine || {
  code=$?
  if [ $code -eq 4 ]; then
    echo "Auth error, exiting"
    exit 1
  elif [ $code -eq 5 ]; then
    echo "Not found, continuing"
  fi
}

# Use in CI/CD pipelines
rex search image myapp --tag production || exit_code=$?
if [ ${exit_code:-0} -ne 0 ] && [ ${exit_code:-0} -ne 5 ]; then
  echo "Error: Failed with code $exit_code"
  exit 1
fi
```

### 2.9 Environment Variables

Rex respects standard environment variables and provides custom variables for
behavior control. Environment variables have lower priority than command-line
flags but higher priority than config file settings.

**Priority Order** (highest to lowest):

1. Command-line flags
2. Environment variables
3. Configuration file (`~/.config/rex/config.toml`)
4. Built-in defaults

#### Standard Environment Variables

**`EDITOR`**:

- Specifies the text editor for `rex config edit`
- Falls back to: `vim` → `nano` → `vi` (in that order)
- Example: `EDITOR=code rex config edit`

**`NO_COLOR`**:

- When set (any value), disables colored output
- Follows <https://no-color.org> convention
- Overridden by explicit `--color` flag
- Example: `NO_COLOR=1 rex image`

**`RUST_LOG`**:

- Controls logging level for rex and dependencies
- Standard Rust logging convention
- Values: `error`, `warn`, `info`, `debug`, `trace`
- Can target specific modules
- Examples:
  - `RUST_LOG=rex=debug rex image`
  - `RUST_LOG=rex::client=trace rex image`
  - `RUST_LOG=trace rex image`

**`HOME`**:

- User's home directory (standard Unix variable)
- Used to resolve `~` in paths
- Default config: `$HOME/.config/rex/config.toml`
- Default cache: `$HOME/.cache/rex/`

**`XDG_CONFIG_HOME`**:

- XDG Base Directory for configuration files
- Default: `~/.config`
- Rex config: `$XDG_CONFIG_HOME/rex/config.toml`

**`XDG_CACHE_HOME`**:

- XDG Base Directory for cache files
- Default: `~/.cache`
- Rex cache: `$XDG_CACHE_HOME/rex/`

**`XDG_STATE_HOME`**:

- XDG Base Directory for state files (logs)
- Default: `~/.local/state`
- Rex logs: `$XDG_STATE_HOME/rex/rex.log`

#### Rex-Specific Environment Variables

**`REX_REGISTRY`**:

- Default registry URL
- Format: `[http://|https://]host[:port]`
- Overridden by: `--registry` flag or current registry in config
- Example: `REX_REGISTRY=https://ghcr.io rex image`

**`REX_CONFIG`**:

- Path to configuration file
- Default: `~/.config/rex/config.toml`
- Overridden by: `--config` flag
- Example: `REX_CONFIG=./custom-config.toml rex image`

**`REX_CACHE_DIR`**:

- Path to cache directory
- Default: `~/.cache/rex/`
- Overridden by: `--cache-dir` flag
- Example: `REX_CACHE_DIR=/tmp/rex-cache rex image`

**`REX_NO_CACHE`**:

- When set (any value), disables caching
- Equivalent to `--no-cache` flag
- Useful for CI/CD environments
- Example: `REX_NO_CACHE=1 rex image`

**`REX_LOG_FORMAT`**:

- Override log format
- Values: `pretty`, `json`
- Default: Auto-detect based on TTY
- Example: `REX_LOG_FORMAT=json rex -vv image`

**`REX_OUTPUT_FORMAT`**:

- Default output format for commands
- Values: `pretty`, `json`, `yaml`
- Default: `pretty` (TTY) or `yaml` (piped)
- Overridden by: `--format` flag
- Example: `REX_OUTPUT_FORMAT=json rex image`

**`REX_COLOR`**:

- Control colored output
- Values: `auto`, `always`, `never`
- Default: `auto`
- Overridden by: `--color` flag or `NO_COLOR`
- Example: `REX_COLOR=never rex image`

**`REX_TIMEOUT`**:

- Request timeout in seconds
- Default: `30`
- Overridden by: `--timeout` flag
- Example: `REX_TIMEOUT=60 rex image`

**`REX_NO_VERIFY_TLS`**:

- When set (any value), skips TLS certificate verification
- Equivalent to `--no-verify-tls` flag
- Security warning: Use only with self-signed certificates
- Example: `REX_NO_VERIFY_TLS=1 rex image`

**`HTTP_PROXY` / `HTTPS_PROXY`**:

- Standard proxy environment variables
- Format: `http://proxy.example.com:8080`
- Automatically respected by HTTP client
- Can include credentials: `http://user:pass@proxy:8080`

**`NO_PROXY`**:

- Comma-separated list of hosts to exclude from proxy
- Example: `NO_PROXY=localhost,127.0.0.1,.internal`

#### Environment Variable Examples

**Development Setup**:

```bash
# Use local registry and disable TLS verification
export REX_REGISTRY=http://localhost:5000
export REX_NO_VERIFY_TLS=1
export REX_CACHE_DIR=/tmp/rex-dev-cache

rex image
```

**CI/CD Pipeline**:

```bash
# Disable cache, use JSON output, increase verbosity
export REX_NO_CACHE=1
export REX_OUTPUT_FORMAT=json
export RUST_LOG=rex=info

rex search image myapp --tag production | jq '.results[0].name'
```

**Debug Session**:

```bash
# Enable debug logging and pretty format logs
export RUST_LOG=rex=debug
export REX_LOG_FORMAT=pretty

rex -vv image inspect myapp:latest
```

**Proxy Environment**:

```bash
# Use corporate proxy
export HTTPS_PROXY=http://proxy.corp.com:8080
export NO_PROXY=localhost,127.0.0.1,registry.internal

rex registry init prod https://registry.example.com
```

**Custom Paths**:

```bash
# Use non-standard XDG directories
export XDG_CONFIG_HOME=/opt/rex/config
export XDG_CACHE_HOME=/opt/rex/cache
export XDG_STATE_HOME=/opt/rex/state

rex config
```
