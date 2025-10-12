# Rex - Registry Explorer Requirements

## Project Overview

Rex is a command-line tool for exploring OCI-compliant container registries
using the OCI Distribution Specification. It provides two operational modes:

1. **CLI Mode**: Non-interactive commands optimized for scripting and automation
2. **TUI Mode**: Interactive terminal user interface for visual exploration

The primary target is Zot registry, with a design that supports any OCI
Distribution Spec compliant registry.

## Core Functionality

### 1. Registry Connection & Authentication

**Registry Protocol:**

- Strict adherence to OCI Distribution Specification
- Support OCI Distribution Spec v1.0+
- HTTPS/TLS connections with certificate verification

**Authentication Methods:**

- **Basic Auth**: Username and password authentication
- **Bearer Token Auth**: OAuth2/token-based authentication
- **Anonymous Access**: For public registries

**Credential Management:**

- Read credentials from Docker config (`~/.docker/config.json`)
- Read credentials from Podman auth (`~/.config/containers/auth.json`)
- Interactive credential prompt when not found
- Secure credential storage for reuse
- Support for per-registry credentials

**Supported Registries:**

- **Primary**: Zot registry (full support)
- **Secondary**: Any OCI Distribution Spec compliant registry
  - Docker Hub
  - GitHub Container Registry (ghcr.io)
  - GitLab Container Registry
  - Harbor
  - Distribution (Docker Registry v2)
  - Others implementing OCI Distribution Spec

### 2. Repository Discovery

**List Repositories:**

- Use OCI Distribution Spec `GET /v2/_catalog` endpoint
- Display repository names
- Handle pagination for large registries
- Show repository count

**Repository Details:**

- Fetch repository metadata
- List all available tags for a repository
- Display tag count
- Support Zot-specific extensions when available

### 3. Image & Tag Inspection

**Tag Listing:**

- Use OCI Distribution Spec `GET /v2/<name>/tags/list` endpoint
- List all tags for a repository
- Handle pagination
- Display tag metadata when available

**Manifest Operations:**

- Fetch manifests using `GET /v2/<name>/manifests/<reference>`
- Support multiple manifest types:
  - OCI Image Manifest
  - OCI Image Index (multi-platform)
  - Docker Image Manifest V2 Schema 2
  - Docker Manifest List
- Display manifest digest (SHA256)
- Parse and show manifest content

**Image Configuration:**

- Retrieve image config blob
- Display configuration details:
  - Architecture and OS
  - Environment variables
  - Entry point and command
  - Working directory
  - Exposed ports
  - Labels
  - Created timestamp
  - Author

**Layer Information:**

- List all layers from manifest
- Display layer digests and sizes
- Show media types
- Calculate total image size
- Display layer URLs (for reference)

**Blob Operations:**

- Support blob retrieval for inspection
- Handle blob redirects
- Verify blob digests

### 4. Multi-Architecture Support

**Image Index Handling:**

- Detect OCI Image Index / Manifest Lists
- Parse platform-specific manifests
- Display available platforms (OS/architecture/variant)
- Allow filtering by platform
- Show platform-specific details

**Platform Information:**

- OS (linux, windows, etc.)
- Architecture (amd64, arm64, arm, etc.)
- Variant (v6, v7, v8, etc.)
- OS version and features

### 5. Output Formatting & Terminal Detection

**Automatic TTY Detection:**

- Detect if stdout is attached to a terminal (TTY)
- **When TTY detected** (interactive terminal):
  - Default output format: **pretty**
  - Enable colored output automatically
  - Show progress bars for long operations
  - Display formatted tables and aligned columns
  - Show status messages and indicators
  - Enable animated spinners
- **When no TTY detected** (piped/redirected output):
  - Default output format: **YAML**
  - Automatically disable colors
  - Suppress progress bars
  - Suppress status messages
  - Output structured data for parsing
  - Optimize for scripting/automation

**Format Options:**

- **Pretty**: Human-readable formatted output with colors and tables
  - Default when stdout is a TTY
  - Tabular layout with aligned columns
  - Color-coded information
- **JSON**: Structured JSON output for machine parsing
  - Explicit selection via `--format json`
  - Compact or pretty-printed based on TTY
- **YAML**: YAML format for configuration tools
  - Default when stdout is NOT a TTY (piped/redirected)
  - Human-readable structured format
  - Easy to parse in scripts
- **Quiet**: Minimal output mode (names/digests only)
  - Suppresses all visual feedback (progress bars, spinners, status messages)
  - Shows only essential output data
  - Different from non-TTY: quiet is explicit, non-TTY is automatic

**Output Customization:**

- `--no-color`: Force disable colored output (overrides TTY detection)
- `--color[=always|never|auto]`: Force color behavior (default: auto)
- `--quiet, -q`: Enable quiet mode (suppress visual feedback)
- Configurable field selection
- Timestamp formatting options (relative or absolute)
- Size formatting (human-readable or bytes)

**Output Behavior Matrix:**

```text
| Mode           | TTY  | Format  | Colors | Progress | Status | Use Case              |
|----------------|------|---------|--------|----------|--------|-----------------------|
| Normal (TTY)   | Yes  | Pretty  | Yes    | Yes      | Yes    | Interactive terminal  |
| Normal (pipe)  | No   | YAML    | No     | No       | No     | Scripting/automation  |
| Quiet (TTY)    | Yes  | Pretty  | Yes    | No       | No     | Minimal feedback      |
| Quiet (pipe)   | No   | YAML    | No     | No       | No     | Script parsing        |
| --no-color     | Yes  | Pretty  | No     | Yes      | Yes    | Plain terminal output |
| --format=json  | Any  | JSON    | No     | No       | No     | JSON output           |
| --format=yaml  | Any  | YAML    | No     | No       | No     | YAML output (forced)  |
```

**Examples:**

```bash
# Interactive terminal - pretty format with colors and progress
rex repos

# Piped to file - YAML format, no colors, no progress
rex repos > repos.yaml

# Explicit JSON format
rex repos --format json | jq '.repositories[] | .name'

# Quiet mode in terminal - pretty format but no progress bars
rex repos -q

# Force pretty format even when piped (for documentation)
rex repos --format pretty | cat
```

### 6. Interactive TUI Features

**Navigation Structure:**

- Registry connection view
- Repository list view
- Tag list view (for selected repository)
- Manifest detail view (for selected tag)
- Layer detail view
- Configuration view

**Navigation Controls:**

- Arrow keys for movement
- Vim-style bindings (hjkl)
- Enter to drill down into items
- Backspace/Esc to navigate back
- Tab to switch between panels
- / for search/filter
- q to quit

**Display Components:**

- Header with registry URL and connection status
- Repository browser with metadata
- Tag list with digest and size
- Manifest viewer (pretty-printed JSON/YAML)
- Layer tree with sizes
- Status bar with shortcuts
- Help overlay

**Interactive Actions:**

- Copy image reference to clipboard (repo:tag)
- Copy digest to clipboard
- View raw manifest
- View image configuration
- Refresh data from registry
- Export manifest to file
- Search/filter repositories and tags
- Sort by name, date, or size

### 7. Zot-Specific Features

**Zot Extensions:**

- Support for Zot's extension APIs when available
- Enhanced metadata display
- Signature verification information (if present)
- Vulnerability scan results (if available)
- Image referrers support
- OCI Artifacts support

**Graceful Degradation:**

- Detect Zot-specific endpoints
- Fall back to standard OCI Distribution Spec if extensions unavailable
- Work seamlessly with non-Zot registries

### 8. Configuration Management

**Configuration File:**

- Location: `~/.config/rex/config.toml`
- Settings:
  - Default registry URL
  - Default output format
  - TUI theme and colors
  - Network timeouts
  - TLS certificate verification options
  - Cache settings
  - Credential storage preferences

**Example Configuration:**

```toml
[default]
registry = "https://localhost:5000"
output_format = "pretty"
verify_tls = true

[tui]
theme = "dark"
vim_bindings = true

[network]
timeout = 30
retry_attempts = 3

[cache]
enabled = true
ttl = 300
```

**Registry Profiles:**

- Define multiple named registry profiles
- Quick switching between registries
- Per-profile authentication

### 9. Error Handling & Diagnostics

**Error Categories:**

- Network errors (connection refused, timeout)
- Authentication errors (401, 403)
- Not found errors (404)
- Rate limiting (429)
- Server errors (5xx)
- Invalid manifest/blob errors
- TLS certificate errors

**Error Reporting:**

- Clear, actionable error messages
- Suggestions for resolution
- Show relevant HTTP status codes
- Display registry error responses

**Debug Mode:**

- Verbose logging with `--verbose` or `-v` flag
- HTTP request/response logging
- Show request headers and body
- Display timing information
- Trace OCI API calls

### 10. Performance Considerations

**Efficient Operations:**

- Connection pooling and reuse
- Parallel requests where appropriate
- Lazy loading of manifests and configs
- Pagination support for large result sets
- Response streaming for large blobs

**Caching Strategy:**

- Optional caching of catalog and tag lists
- Short-lived cache with TTL
- Cache invalidation options
- Memory-based cache for TUI mode

## Non-Functional Requirements

### Usability

- Zero-config for local Zot registry (http://localhost:5000)
- Auto-detect Docker/Podman credentials
- Intuitive command structure
- Comprehensive help and examples
- Progress indicators for long operations

### Performance

- Repository listing under 2 seconds (local network)
- Responsive TUI with no lag
- Efficient memory usage
- Support registries with 10,000+ repositories
- Concurrent requests where appropriate

### Compatibility

- Cross-platform: Linux, macOS, Windows
- OCI Distribution Spec v1.0+ compliance
- Work with any spec-compliant registry
- Handle both HTTP and HTTPS
- IPv4 and IPv6 support

### Security

- Secure credential storage using OS keychain
- No plaintext password storage in config
- TLS/SSL certificate verification by default
- Warning for self-signed certificates
- Support for custom CA certificates
- No credential logging in verbose mode

### Reliability

- Graceful error handling
- Network failure recovery
- Rate limit backoff
- Timeout configuration
- Retry logic for transient failures

## Success Criteria

1. Successfully connect to Zot registry and list repositories
2. Authenticate using basic auth and bearer token
3. Inspect multi-platform images and display all platforms
4. TUI provides smooth navigation through registry hierarchy
5. Output formats (pretty, JSON, YAML) work correctly
6. Works with Docker Hub and GitHub Container Registry
7. Clear error messages for common failure scenarios
8. Complete operations within performance targets

## Future Enhancements (Post-MVP)

Out of scope for MVP but may be added in future releases:

- Image pull/push operations
- Tag/manifest deletion (DELETE endpoints)
- Image copying between registries
- Vulnerability scanning integration
- Signature verification (Cosign, Notation)
- OCI Artifact inspection (Helm charts, WASM, etc.)
- Image referrers API support
- Registry statistics and analytics
- Webhook configuration
- Garbage collection triggers
- Storage quota information
- Automated tag cleanup
- Image diff/comparison
- SBOM and attestation viewing
- GraphQL API support (Zot-specific)
