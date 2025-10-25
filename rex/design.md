# Rex - CLI and TUI Design

## Overview

Rex is a command-line and terminal UI tool for exploring OCI-compliant
container registries. It is built on top of the librex library and provides two
interaction modes:

1. **CLI Mode**: Fast, scriptable command-line interface
2. **TUI Mode**: Interactive terminal UI for visual exploration

## Architecture

```text
┌─────────────────────────────────────────┐
│         CLI Mode (rex)                   │
│  - Argument parsing (clap)              │
│  - Output formatting                    │
│  - Piping & scripting                   │
└─────────────────────────────────────────┘
              │
              │  both use
              ▼
┌─────────────────────────────────────────┐
│       Core Engine (librex)              │
│  - Registry operations                   │
│  - Authentication                       │
│  - Caching                              │
│  - Search                               │
└─────────────────────────────────────────┘
              ▲
              │
┌─────────────────────────────────────────┐
│         TUI Mode (rex tui)              │
│  - Event handling (crossterm)           │
│  - UI rendering (ratatui)               │
│  - Interactive navigation               │
└─────────────────────────────────────────┘
```

## Design Principle

**Single Entry Point**: Both CLI and TUI modes are provided by the same `rex` binary. This ensures:

- Single installation process
- Consistent configuration
- Shared cache
- Easy discovery (TUI shown in `--help`)
- Natural workflow between modes

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

#### `rex registry cache sync`

Pre-populate cache by fetching and caching registry metadata.

**Command**: `rex registry cache sync [NAME] [OPTIONS]`

**Arguments**:

- `[NAME]`: Registry name (optional, uses default if omitted)

**Options**:

- `--manifests`: Also fetch and cache image manifests (increases cache size)
- `--all`: Sync cache for all registries
- `--force`: Re-fetch even if entries exist in cache

**Behavior**:

- Fetches catalog (list of repositories)
- Fetches tags for each repository
- Optionally fetches manifests for each tag
- Populates cache for faster subsequent operations
- Shows progress during sync

**Examples**:

```bash
# Sync cache for current registry
rex registry cache sync

# Sync cache for specific registry
rex registry cache sync prod

# Sync all registries
rex registry cache sync --all

# Sync with manifests (slower, more complete)
rex registry cache sync --manifests

# Force re-sync even if cached
rex registry cache sync --force
```

**Output**:

```text
Syncing cache for 'local' (http://localhost:5000)...

Fetching catalog... ✓ (45 repositories)
Fetching tags... ✓ (312 tags across 45 repositories)

Cache synced successfully:
  45 catalog entries
  312 tag entries
  Total size: 2.8 MB

Cache location: ~/.cache/rex/http___localhost_5000/
```

**Output** (with --manifests):

```text
Syncing cache for 'local' (http://localhost:5000)...

Fetching catalog... ✓ (45 repositories)
Fetching tags... ✓ (312 tags across 45 repositories)
Fetching manifests... ✓ (312 manifests)

Cache synced successfully:
  45 catalog entries
  312 tag entries
  312 manifest entries
  Total size: 24.5 MB

Cache location: ~/.cache/rex/http___localhost_5000/
```

**Use Cases**:

- Pre-populate cache before going offline
- Warm up cache after clearing
- Prepare for faster browsing in TUI mode
- Mirror metadata for backup purposes

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

#### `rex image list`

List all images in the registry.

**Command**: `rex image list [OPTIONS]`

**Aliases**: `rex image ls`

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
rex image list

# Using alias
rex image ls

# List with filter
rex image list --filter alpine

# List in JSON format
rex image list --format json

# List only names (quiet mode)
rex image list --quiet
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

#### `rex image tags`

List all tags for a specific image.

**Command**: `rex image tags <NAME> [OPTIONS]`

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
rex image tags alpine

# List with sorting by date (newest first)
rex image tags alpine --sort date

# Filter tags
rex image tags myapp --filter v1

# Show only tag names
rex image tags alpine --quiet
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

#### `rex image show`

Show details for a specific image tag.

**Command**: `rex image show <NAME>:<TAG> [OPTIONS]`

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
rex image show alpine:latest

# Show for specific platform
rex image show alpine:latest --platform linux/arm64

# Show using digest
rex image show alpine@sha256:c5b1261d...

# JSON output
rex image show alpine:latest --format json
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

### 2.6 Search Command

Search for images and tags across the registry using fuzzy matching in a unified interface.

#### `rex search`

Unified search command that searches both repository names and tags simultaneously.

**Command**: `rex search <QUERY> [OPTIONS]`

**Arguments**:

- `<QUERY>`: Search query (supports fuzzy matching)

**Options**:

- `--format, -f <FORMAT>`: Output format (`pretty`, `json`, `yaml`)
- `--limit <N>`: Limit number of results per category (default: 50)

**Behavior**:

- Fuzzy searches both repository names and tags across all images
- Returns results in two sections: Images (repositories) and Tags
- Tags are shown with their full reference (image:tag)
- Results are ordered from best match to least relevant within each section
- Only shows matches with >= 50% accuracy score
- Scores are calculated internally but not displayed

**Examples**:

```bash
# Search for anything matching "nginx"
rex search nginx

# Limit results per category
rex search alpine --limit 10

# JSON output
rex search nginx --format json
```

**Output** (pretty format):

```bash
rex search nginx
```

```text
Images:
  nginx
  nginx-proxy
  my-nginx

Tags:
  webapp:nginx-1.21
  alpine:nginx-latest
  backend:nginx-base
```

**Output** (JSON format):

```json
{
  "query": "nginx",
  "images": {
    "total_results": 3,
    "results": [
      {
        "name": "nginx"
      },
      {
        "name": "nginx-proxy"
      },
      {
        "name": "my-nginx"
      }
    ]
  },
  "tags": {
    "total_results": 3,
    "results": [
      {
        "image": "webapp",
        "tag": "nginx-1.21",
        "reference": "webapp:nginx-1.21"
      },
      {
        "image": "alpine",
        "tag": "nginx-latest",
        "reference": "alpine:nginx-latest"
      },
      {
        "image": "backend",
        "tag": "nginx-base",
        "reference": "backend:nginx-base"
      }
    ]
  }
}
```

**Output** (YAML format):

```yaml
query: nginx
images:
  total_results: 3
  results:
    - name: nginx
    - name: nginx-proxy
    - name: my-nginx
tags:
  total_results: 3
  results:
    - image: webapp
      tag: nginx-1.21
      reference: webapp:nginx-1.21
    - image: alpine
      tag: nginx-latest
      reference: alpine:nginx-latest
    - image: backend
      tag: nginx-base
      reference: backend:nginx-base
```

**Match Accuracy Threshold**:

- Results are internally scored based on match quality
- Only matches with >= 50% accuracy are displayed
- Results are ordered from best match (highest score) to worst match
  (lowest score above threshold) within each section
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

### 2.8 Shell Completion

Rex supports generating shell completion scripts for multiple shells, enabling
tab completion for commands, subcommands, and options.

#### `rex completion <SHELL>`

Generate shell completion script for the specified shell.

**Command**: `rex completion <SHELL>`

**Arguments**:

- `<SHELL>`: Target shell (required)
  - `bash`: Bash shell
  - `zsh`: Zsh shell
  - `fish`: Fish shell
  - `powershell`: PowerShell
  - `elvish`: Elvish shell

**Behavior**:

- Generates completion script and writes to stdout
- Script should be sourced or installed according to shell conventions
- Enables tab completion for rex commands and options
- Returns exit code 0

**Examples**:

```bash
# Generate bash completion
rex completion bash > ~/.local/share/bash-completion/completions/rex

# Generate zsh completion
rex completion zsh > ~/.zfunc/_rex

# Generate fish completion
rex completion fish > ~/.config/fish/completions/rex.fish

# Generate PowerShell completion
rex completion powershell > rex.ps1
```

**Installation**:

**Bash**:
```bash
rex completion bash | sudo tee /usr/share/bash-completion/completions/rex
```

**Zsh**:
```bash
rex completion zsh > ~/.zfunc/_rex
# Add to .zshrc: fpath+=~/.zfunc
```

**Fish**:
```bash
rex completion fish > ~/.config/fish/completions/rex.fish
```

### 2.9 Output Formatting System

Rex implements a comprehensive TTY-aware formatting system that automatically
adapts output based on whether stdout/stderr is a terminal or being piped.

#### Architecture

The formatting system uses a trait-based design with two implementations:

**`OutputFormatter` Trait**:
```rust
pub trait OutputFormatter: Send + Sync {
    fn success(&self, message: &str);
    fn error(&self, message: &str);
    fn warning(&self, message: &str);
    fn spinner(&self, message: &str) -> ProgressBar;
    fn progress_bar(&self, len: u64, message: &str) -> ProgressBar;
    fn finish_progress(&self, pb: ProgressBar, message: &str);
    fn checkmark(&self) -> String;
}
```

**Implementations**:

1. **`TtyFormatter`**: Used when stdout or stderr is a terminal
   - Colored output (green ✓, red ✗, yellow ⚠)
   - Animated spinners for indeterminate operations
   - Progress bars with [████░░] visualization and ETA
   - Uses `owo-colors` for coloring and `indicatif` for progress

2. **`PlainFormatter`**: Used when output is piped or redirected
   - Plain text output without ANSI codes
   - Compatible with grep, sed, awk, and other text processing tools
   - Hidden progress bars (returns immediately)
   - Suitable for CI/CD, logging, and scripting

#### TTY Detection

The formatter is selected automatically based on:

1. **NO_COLOR environment variable**: If set, always use plain formatting
2. **TTY detection**: Checks if stdout OR stderr is a terminal
   - Uses `std::io::IsTerminal` trait
   - Checks both streams since errors go to stderr

```rust
pub fn create_formatter() -> Box<dyn OutputFormatter> {
    if std::env::var("NO_COLOR").is_ok() {
        return Box::new(PlainFormatter);
    }
    if std::io::stdout().is_terminal() || std::io::stderr().is_terminal() {
        Box::new(TtyFormatter)
    } else {
        Box::new(PlainFormatter)
    }
}
```

#### Output Functions

**Success Messages**:
```rust
format::success("Registry initialized successfully");
// TTY:   ✓ Registry initialized successfully  (green ✓)
// Pipe:  ✓ Registry initialized successfully  (plain text)
```

**Error Messages**:
```rust
format::error("Failed to connect to registry");
// TTY:   ✗ Failed to connect to registry  (red ✗)
// Pipe:  ✗ Failed to connect to registry  (plain text)
```

**Warning Messages**:
```rust
format::warning("Cache is stale");
// TTY:   ⚠ Cache is stale  (yellow ⚠)
// Pipe:  ⚠ Cache is stale  (plain text)
```

**Progress Indicators**:

```rust
let formatter = format::create_formatter();

// Spinner for indeterminate operations
let spinner = formatter.spinner("Fetching catalog...");
// ... perform operation ...
formatter.finish_progress(spinner, "Fetched catalog (45 repositories)");

// Progress bar for determinate operations
let pb = formatter.progress_bar(100, "Downloading layers");
for i in 0..100 {
    // ... process item ...
    pb.inc(1);
}
formatter.finish_progress(pb, "Downloaded all layers");
```

**TTY Output**:
```text
⠋ Fetching catalog...
✓ Fetched catalog (45 repositories)

Downloading layers [████████████████░░░░] 80/100 (2s)
✓ Downloaded all layers
```

**Piped Output**:
```text
Fetching catalog...
✓ Fetched catalog (45 repositories)
Downloading layers (0/100)
✓ Downloaded all layers
```

#### Symbol Functions

For data display in formatted output (like `RegistryCheckResult`), use:

```rust
// Colored/plain checkmark based on TTY
let check = format::checkmark();
// TTY:   ✓ (green)
// Pipe:  ✓ (plain)

// Colored/plain error mark based on TTY
let cross = format::error_mark();
// TTY:   ✗ (red)
// Pipe:  ✗ (plain)
```

#### Usage Examples

**Registry Sync with Progress**:
```bash
# TTY output (terminal)
$ rex registry cache sync
⠋ Fetching catalog...
✓ Fetched catalog (45 repositories)
Fetching tags [████████████████░░░░] 36/45 (5s)
✓ Fetched 312 tags across 45 repositories

# Piped output (script)
$ rex registry cache sync | cat
Fetching catalog...
✓ Fetched catalog (45 repositories)
Fetching tags (0/45)
✓ Fetched 312 tags across 45 repositories
```

**Error Handling**:
```bash
# TTY output
$ rex config set invalid.key value
✗ Unknown config key: invalid.key

# Piped output (same, but no colors)
$ rex config set invalid.key value 2>&1 | cat
✗ Unknown config key: invalid.key
```

#### Dependencies

The formatting system uses:

- **`indicatif`** v0.17: Progress bars and spinners
- **`owo-colors`** v4: Terminal coloring with automatic TTY detection
- **`clap_complete`** v4.5: Shell completion generation

#### Design Rationale

**Why Trait-Based?**
- Clean separation of TTY vs non-TTY formatting logic
- No scattered `is_tty()` checks throughout codebase
- Easy to test each implementation independently
- Single decision point at application startup

**Why Check Both stdout and stderr?**
- Success messages go to stdout (`println!`)
- Error messages go to stderr (`eprintln!`)
- User might redirect only one stream
- Ensures colors appear if either is a terminal

**Why NO_COLOR Takes Precedence?**
- Respects user preference over automatic detection
- Follows [NO_COLOR standard](https://no-color.org/)
- Important for accessibility (screen readers, vision impairments)

### 2.10 Exit Codes

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

### 2.11 Environment Variables

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

- When set (any value), disables colored output and uses plain formatting
- Follows <https://no-color.org> convention
- Affects the output formatting system (see section 2.9)
- Takes precedence over TTY detection
- Important for accessibility (screen readers, vision impairments)
- Example: `NO_COLOR=1 rex registry check`

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

---

## Part 3: TUI Mode Design

The TUI (Terminal User Interface) mode provides an interactive, visual way to
explore container registries. It is launched with `rex tui` and provides a rich
interface for browsing images, tags, and viewing details.

### 3.1 Launching TUI Mode

**Command**: `rex tui [OPTIONS] [VIEW]`

**Arguments**:

- `[VIEW]`: Optional starting view or resource to focus on

**Options**:

- `--registry, -r <NAME>`: Start with specific registry selected
- `--theme <THEME>`: Override theme (`dark`, `light`)

**Examples**:

```bash
# Launch TUI at main view
rex tui

# Launch TUI focused on specific image
rex tui alpine:latest

# Launch with specific registry
rex tui --registry prod

# Launch with light theme
rex tui --theme light
```

### 3.2 TUI Architecture

**Framework**: ratatui (terminal UI library)
**Event Handling**: crossterm (terminal event handling)

**Component Structure**:

```text
rex/src/tui/
├── mod.rs              # TUI entry point and main loop
├── app.rs              # Application state machine
├── ui/                 # UI components
│   ├── mod.rs
│   ├── images.rs       # Images list view
│   ├── tags.rs         # Tags list view
│   ├── details.rs      # Image details panel
│   ├── registry.rs     # Registry selector
│   └── help.rs         # Help panel
├── events.rs           # Event handling and keybindings
└── theme.rs            # Color themes and styling
```

### 3.3 Main Views

**Images View** (default starting view):

- List of all images (repositories) in registry
- Search/filter box at top
- Shows: image name, tags count, last updated
- Navigate with arrow keys or vim bindings (j/k)
- Press Enter to view tags for selected image

**Tags View**:

- List of tags for selected image
- Breadcrumb shows: registry > image
- Shows: tag name, digest (truncated), size, platforms
- Press Enter to view details for selected tag

**Details View**:

- Full details for selected image:tag
- Shows: manifest info, configuration, layers, history
- Scrollable content
- Press 'b' to go back

**Registry Selector**:

- List of configured registries
- Shows: name, URL, connection status
- Switch between registries without exiting TUI
- Press 'r' to open registry selector

### 3.4 Keybindings

**Navigation** (Standard Mode):

- `↑`/`k`: Move up
- `↓`/`j`: Move down
- `←`/`h`: Go back / Previous view
- `→`/`l`/`Enter`: Select / Next view
- `Page Up`/`Page Down`: Scroll page
- `Home`/`End`: Jump to top/bottom
- `Tab`: Switch between panels

**Actions**:

- `/`: Focus search box
- `r`: Open registry selector
- `i`: Inspect selected image (full details)
- `y`: Copy selected item to clipboard
- `R`: Refresh current view (bypass cache)
- `?`: Toggle help panel
- `q`/`Esc`: Go back or quit

**Vim Mode** (when enabled):

- All standard bindings work
- Plus vim-style: `gg` (top), `G` (bottom), `Ctrl+d`/`Ctrl+u` (half page)

### 3.5 Search and Filtering

**Real-time Search**:

- Type `/` to focus search box
- Results filter as you type
- Fuzzy matching like CLI
- Press Enter to close search and navigate results
- Press Esc to cancel search

**Visual Feedback**:

- Matching characters highlighted
- Result count shown
- "No matches" message when nothing found

### 3.6 Theme and Styling

**Dark Theme** (default):

- Background: Dark gray/black
- Text: Light gray/white
- Highlights: Blue/Cyan
- Borders: Gray
- Selected: Bright blue background

**Light Theme**:

- Background: White/light gray
- Text: Dark gray/black
- Highlights: Dark blue
- Borders: Gray
- Selected: Light blue background

**Configuration**:

```toml
[tui]
theme = "dark"  # or "light"
vim_bindings = true
```

### 3.7 Status Bar

Always visible at bottom:

```text
[CURRENT_VIEW] | Registry: localhost:5000 | Image: alpine | [HELP: ?] [QUIT: q]
```

Shows:

- Current view name
- Current registry
- Current context (image, tag if applicable)
- Quick help reminders

### 3.8 Performance Considerations

**Async Loading**:

- TUI doesn't block on network requests
- Shows loading spinner while fetching
- Uses cache aggressively
- Background refresh

**Pagination**:

- Large lists paginated automatically
- Load more as user scrolls
- Prevents UI lag with thousands of items

**Cache Strategy**:

- Serve from cache immediately
- Trigger background refresh
- Update UI when fresh data arrives (eventual consistency)

---

## Part 4: Cache Management

Rex uses a sophisticated two-tier caching system to minimize network requests
and improve performance across both CLI and TUI modes.

### 4.1 Cache Architecture

**Two-Tier Design**:

1. **L1 Cache (Memory)**:
   - In-memory LRU cache
   - Capacity: 1000 entries or 100MB
   - Access time: <1μs
   - Cleared on process exit

2. **L2 Cache (Disk)**:
   - Persistent filesystem storage
   - Location: `~/.cache/rex/`
   - Access time: 1-10ms
   - Survives process restarts

**Cache Flow**:

```text
Request → L1 Check → L2 Check → Network Fetch
            ↓           ↓            ↓
          Return      Hydrate L1   Store in L1+L2
```

### 4.2 Per-Registry Cache Isolation

Each registry gets its own cache directory to prevent conflicts:

```text
~/.cache/rex/
├── http___localhost_5000/          # Local Zot registry
│   ├── catalog                      # Repository list
│   ├── alpine/
│   │   ├── _tags                    # Tags for alpine
│   │   ├── tags/
│   │   │   └── latest/
│   │   │       └── manifest         # Cached manifest
│   │   └── manifests/
│   │       └── sha256_abc123...     # Digest-addressed manifest
│   └── nginx/
│       └── _tags
└── https___registry_example_com/   # Production registry
    └── ...
```

**Directory Naming**:

- Registry URL sanitized: replace `://`, `/`, `:`, `.` with `_`
- Example: `https://registry.example.com` → `https___registry_example_com`

### 4.3 Cache Entry Format

Each cache entry stores:

```rust
struct CacheEntry {
    data: Vec<u8>,              // Serialized data
    cached_at: SystemTime,       // When cached
    ttl: Duration,               // Time-to-live
}
```

Serialization: `bincode` (fast, compact binary format)

### 4.4 What Gets Cached

**1. Repository Catalog**:

- Key: `catalog`
- TTL: 300 seconds (5 minutes)
- Reason: Repositories change moderately
- Size: ~1-10KB

**2. Tag Lists**:

- Key: `{repository}/_tags`
- TTL: 300 seconds (5 minutes)
- Reason: Tags are mutable
- Size: ~1-5KB per image

**3. Manifests (by tag)**:

- Key: `{repository}/tags/{tag}/manifest`
- TTL: 600 seconds (10 minutes)
- Reason: Tags can be reassigned
- Size: ~1-10KB

**4. Manifests (by digest)**:

- Key: `{repository}/manifests/{digest}`
- TTL: 86400 seconds (24 hours)
- Reason: Digest-addressed content is immutable
- Size: ~1-10KB

**5. Image Configs**:

- Currently NOT cached at L2 (fetched fresh)
- Could be cached in future with digest-based key

**NOT Cached**:

- Blob/layer content (too large: GBs)
- Authentication tokens (handled by auth module)

### 4.5 TTL Strategy

Different data types have different freshness requirements:

| Type | TTL | Reason |
|------|-----|--------|
| Catalog | 5 min | Repos added/removed moderately |
| Tags | 5 min | Tags created/deleted frequently |
| Manifest (tag) | 10 min | Tags can point to different digests |
| Manifest (digest) | 24 hours | Immutable, but verify occasionally |
| Config (digest) | 24 hours | Immutable, but verify occasionally |

**Configurable**:

```toml
[cache.ttl]
catalog = 300        # seconds
tags = 300
manifest = 600
config = 3600
```

### 4.6 Cache Operations Flow

**Example: `rex image list`**

```text
1. User runs: rex image list

2. Rex CLI calls: list_images(registry_url, filter, limit)

3. list_images() calls:
   cache_dir = get_registry_cache_dir(registry_url)
   // → ~/.cache/rex/http___localhost_5000/

4. Build librex::Rex with cache:
   Rex::builder()
       .registry_url(registry_url)
       .with_cache(cache_dir)  ← Cache enabled here
       .build()

5. Call: rex.list_repositories().await

6. librex::Registry::list_repositories():
   cache_key = "catalog"
   
   // Check L1 memory cache
   if cache.memory.get(cache_key) → HIT
       return cached_data  ✓ (< 1μs)
   
   // Check L2 disk cache
   if cache.disk.read(cache_key) → HIT
       cache.memory.put(cache_key, data)  // Hydrate L1
       return cached_data  ✓ (2-5ms)
   
   // Network fetch
   data = client.fetch_catalog()  // HTTP GET /v2/_catalog
   
   // Store in cache
   cache.memory.put(cache_key, data)
   cache.disk.write(cache_key, data)
   
   return data  ✓ (100-500ms)
```

### 4.7 Cache Coherence

**Weak Consistency** (default):

- Serve cached data until TTL expires
- No validation with registry
- Fast, suitable for most operations

**Strong Consistency** (`--no-cache` flag):

- Bypass cache entirely
- Always fetch fresh from network
- Guarantees up-to-date data
- Used when freshness is critical

**Eventual Consistency** (TUI mode):

- Serve stale data immediately
- Trigger background refresh
- Update UI when fresh data arrives
- Provides fast UI responsiveness

### 4.8 Cache Eviction

**TTL-Based** (automatic):

- Expired entries removed on read
- Periodic cleanup on startup

**LRU** (memory cache):

- Least recently used evicted when capacity reached
- Ensures bounded memory usage

**Size-Based** (disk cache):

- When limit exceeded (default: 1GB)
- Evict oldest entries first
- Prioritize immutable entries over mutable

**Manual**:

- `rex registry cache clear` - Clear all
- `rex registry cache prune` - Remove expired only

### 4.9 Cache Management Commands

**View Statistics**:

```bash
rex registry cache stats
```

Output:

```text
Cache Statistics for 'local' (http://localhost:5000)

Overview:
  Total Entries: 245
  Total Size: 12.3 MB
  Hit Rate: 87.4%

By Type:
  Catalogs:   5 entries, 45 KB
  Tags:       28 entries, 156 KB
  Manifests:  142 entries, 8.2 MB
```

**Clear Cache**:

```bash
# Clear cache for current registry
rex registry cache clear

# Clear all caches
rex registry cache clear --all

# Clear specific type
rex registry cache clear --type manifest
```

**Prune Expired**:

```bash
# Remove expired entries
rex registry cache prune

# Dry run (show what would be removed)
rex registry cache prune --dry-run
```

### 4.10 Cache Interaction Example

**Scenario**: User inspects an image

```text
$ rex image inspect alpine:latest

1. CLI determines cache dir:
   ~/.cache/rex/http___localhost_5000/

2. Build Rex with cache enabled

3. Fetch manifest (cached):
   Key: alpine/tags/latest/manifest
   ✓ L1 MISS
   ✓ L2 HIT (2ms)
   → Hydrate L1

4. Parse manifest to get config digest:
   sha256:abc123...

5. Fetch config blob (NOT cached):
   HTTP GET /v2/alpine/blobs/sha256:abc123...
   → Network request (150ms)

6. Parse config and extract fields

7. Display complete image inspection

Total time: ~152ms (vs ~300ms without cache)
```

**Second run** (within TTL):

```text
$ rex image inspect alpine:latest

1. Same cache dir

2. Fetch manifest:
   ✓ L1 HIT (<1ms)
   
3. Fetch config blob:
   → Network request (150ms)
   
Total time: ~150ms
```

### 4.11 Cache Configuration

Users can customize cache behavior:

```toml
[cache]
enabled = true

[cache.ttl]
catalog = 300        # 5 minutes
tags = 300          # 5 minutes
manifest = 600      # 10 minutes
config = 3600       # 1 hour

[cache.limits]
memory_entries = 1000
memory_size_mb = 100
disk_entries = 10000
disk_size_mb = 1024

[cache.behavior]
consistency = "weak"     # weak, strong, eventual
serve_stale = true       # Serve expired data while refreshing
```

### 4.12 Cache Benefits

**Performance**:

- 50-95% reduction in network requests
- Sub-millisecond response for cached data
- Reduced registry load

**Offline Support**:

- Work with cached data when network unavailable
- TUI remains responsive with cached data

**Cost Savings**:

- Fewer requests to rate-limited/paid registries
- Reduced bandwidth usage

**User Experience**:

- Instant responses for repeated operations
- TUI feels snappy and responsive
- CLI commands complete faster

---

## Summary

Rex provides both CLI and TUI modes powered by the librex library:

- **CLI Mode**: Fast, scriptable commands for automation
- **TUI Mode**: Interactive visual exploration
- **Shared Cache**: Two-tier caching (L1 memory + L2 disk)
- **Per-Registry Isolation**: Each registry has separate cache
- **Configurable**: TTLs, limits, and behavior customizable
- **Transparent**: Caching is automatic and invisible to user

The design ensures optimal performance while maintaining data freshness and
providing flexibility for different use cases.
