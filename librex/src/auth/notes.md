# Auth Module - Implementation Notes

## Overview

The auth module provides authentication support for OCI-compliant container registries. It implements the OCI Distribution Specification authentication flow, supporting anonymous access, HTTP Basic authentication, and Bearer token authentication.

## Current Implementation Status

### Phase 1: Core Types ✓ (Completed)

- **Credentials enum**: Represents different authentication methods
  - `Anonymous`: No authentication required
  - `Basic { username, password }`: HTTP Basic authentication with base64 encoding
  - `Bearer { token }`: OAuth2-style bearer token authentication
  - `to_header_value()`: Generates Authorization header values

- **AuthChallenge struct**: Parses WWW-Authenticate headers from 401 responses
  - Extracts: scheme, realm, service, scope
  - Supports both Bearer and Basic authentication challenges
  - Validates required fields (realm must be present)

### Phase 2: Token Flow (Pending)

The full Bearer token challenge/response flow needs to be implemented:

1. Attempt request without authentication
2. Receive 401 with WWW-Authenticate header
3. Parse challenge to extract realm, service, and scope
4. Request token from authentication service at the realm URL
5. Include credentials (if available) in token request
6. Parse token response (JSON with `token` or `access_token` field)
7. Retry original request with Bearer token in Authorization header
8. Handle token expiration (some responses include `expires_in`)

**Example token request:**
```http
GET /token?service=registry.example.com&scope=repository:alpine:pull HTTP/1.1
Host: auth.example.com
Authorization: Basic <base64-encoded-credentials>
```

**Example token response:**
```json
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 300,
  "issued_at": "2024-01-15T10:30:00Z"
}
```

**Implementation needs:**
- `AuthManager` or `TokenFetcher` struct to orchestrate the flow
- HTTP client integration to fetch tokens
- Token response parsing (handle both `token` and `access_token` fields)

### Phase 3: Token Caching (Pending)

To avoid repeated authentication, tokens should be cached:

- **Cache key**: `(registry, service, scope)` tuple
- **Cache value**: Token + expiration time
- **Storage**: In-memory HashMap (for now)
- **Eviction**: Remove expired tokens, clear on auth failures

**Structure:**
```rust
struct TokenCache {
    tokens: HashMap<(String, String, String), CachedToken>,
}

struct CachedToken {
    token: String,
    expires_at: Option<SystemTime>,
}
```

### Phase 4: Credential Store ✓ (Completed)

**Current Implementation:**

A trait-based credential storage abstraction with a file-based implementation:

```rust
pub trait CredentialStore {
    fn store(&mut self, registry: &str, credentials: &Credentials) -> Result<()>;
    fn get(&self, registry: &str) -> Result<Option<Credentials>>;
    fn remove(&mut self, registry: &str) -> Result<()>;
    fn list(&self) -> Result<Vec<String>>;
}
```

**FileCredentialStore:**
- Stores credentials in TOML format at `~/.config/rex/credentials.toml`
- Sets file permissions to 0600 (user read/write only) on Unix
- Base64 encodes passwords for basic obfuscation
- Supports multiple registries in a single file
- Credentials persist across process restarts

**Format:**
```toml
["registry.example.com"]
username = "user"
password = "cGFzc3dvcmQ="  # base64 encoded
```

**Security:**
- File permissions restricted to user only (Unix)
- Basic encoding (not encryption) - relies on filesystem permissions
- TODO: Add OS keyring integration for production use

**Future Enhancements:**
1. Docker config file reader (`~/.docker/config.json`)
2. Podman auth file reader (`~/.config/containers/auth.json`)
3. OS-specific secure storage:
   - macOS: Keychain
   - Linux: Secret Service API (libsecret)
   - Windows: Credential Manager
4. Configuration option to choose storage backend

## Architecture Decisions

### Why Basic Types First?

We implemented the core types (`Credentials`, `AuthChallenge`) first because they are:
- Self-contained with no external dependencies (except base64)
- Testable without requiring HTTP clients or real registries
- Needed by both the token flow and credential store implementations

### Incremental Implementation Strategy

Following the same TDD approach as the client module:
1. **Phase 1**: Core types (credentials, challenge parsing) ✓
2. **Phase 4**: Credential store (file-based storage) ✓
3. **Phase 2**: Token challenge/response flow (when needed)
4. **Phase 3**: Token caching (performance optimization)

This allows us to:
- Use the auth module immediately for basic use cases
- Store and retrieve credentials for login/logout commands
- Add complexity only when needed
- Test each phase independently
- Avoid over-engineering

**Why Phase 4 before Phase 2?**
We implemented credential storage before the token flow because:
- Login/logout commands need storage immediately
- Storage is simpler and self-contained
- Token flow can use stored credentials once implemented

## Security Considerations

### Current Implementation

- **Password Handling**: Passwords are held in memory as Strings
  - ⚠️ Rust Strings are not zeroed on drop by default
  - Future: Consider using `zeroize` crate for sensitive data

- **Base64 Encoding**: Used for Basic auth header generation
  - Not encryption, just encoding (security depends on HTTPS)

- **No Credential Storage**: Currently no persistence of credentials
  - Credentials must be provided for each session
  - Future: Will integrate with OS keychain for secure storage

### Future Security Enhancements

1. **Secure Memory**:
   - Use `zeroize` crate to clear passwords from memory
   - Use `secstr` crate for password strings

2. **Credential Storage**:
   - Never store plaintext passwords in files
   - Use OS keychain/credential manager when available:
     - macOS: Keychain
     - Linux: Secret Service API (libsecret)
     - Windows: Credential Manager
   - Fall back to Docker/Podman config files (base64-encoded)

3. **Token Security**:
   - Clear expired tokens from cache
   - Don't log tokens or credentials
   - Mask sensitive data in error messages

4. **HTTPS Enforcement**:
   - Warn when using HTTP instead of HTTPS
   - Consider refusing Basic auth over HTTP

## Testing Strategy

### Current Tests (22 tests)

- **Credentials** (9 tests):
  - Creation methods (anonymous, basic, bearer)
  - Header value generation
  - Proper formatting
  - AuthChallenge parsing (Bearer, Basic, error cases)

- **FileCredentialStore** (13 tests):
  - Store and retrieve Basic credentials
  - Get non-existent credentials returns None
  - Remove credentials
  - List all registries with credentials
  - Persistence across instances (save/load from file)
  - Overwrite existing credentials
  - Multiple registries support
  - Anonymous credentials rejection
  - Bearer token storage not yet supported
  - File permissions (0600 on Unix)
  - Parent directory creation
  - Empty store behavior

### Future Tests

- **Token Flow**:
  - Mock HTTP responses for token endpoints
  - Handle different token response formats
  - Token expiration and refresh
  - Error handling (invalid credentials, network errors)

- **Token Cache**:
  - Cache hit/miss behavior
  - Expiration handling
  - Cache key generation

- **Credential Store**:
  - Parse Docker config files
  - Parse Podman config files
  - Decode base64 credentials
  - Handle missing files gracefully

## Integration with Client Module

The auth module will eventually be used by the client module (or a higher-level registry client) to handle 401 responses:

```rust
// Pseudo-code for future integration
async fn authenticated_request(client: &Client, url: &str, creds: &Credentials) -> Result<Response> {
    // 1. Try request without auth (or with cached token)
    let response = client.get(url).send().await?;

    // 2. If 401, parse challenge and get token
    if response.status() == 401 {
        let challenge = AuthChallenge::parse(response.headers())?;
        let token = fetch_token(&challenge, creds).await?;

        // 3. Retry with token
        let bearer = Credentials::Bearer { token };
        return client.get(url)
            .header("Authorization", bearer.to_header_value().unwrap())
            .send()
            .await;
    }

    Ok(response)
}
```

## Dependencies

### Current
- `base64 = "0.22"` - For Basic authentication header encoding
- `toml = "0.9.8"` - For credential file serialization
- `serde` - For credential serialization/deserialization

### Future
- HTTP client (already have reqwest in client module)
- `serde_json` - For parsing Docker config and token responses
- `zeroize` (optional) - For secure password handling
- Platform-specific keychain libraries (optional):
  - `keyring` crate (cross-platform wrapper)
  - Or platform-specific crates

## References

- [OCI Distribution Spec - Authentication](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#authentication)
- [Docker Registry Token Authentication](https://docs.docker.com/registry/spec/auth/token/)
- [RFC 7617 - HTTP Basic Authentication](https://tools.ietf.org/html/rfc7617)
- [RFC 6750 - OAuth 2.0 Bearer Token](https://tools.ietf.org/html/rfc6750)

## Future Enhancements

1. **Automatic Token Refresh**: Detect token expiration before making requests
2. **Persistent Token Cache**: Store tokens on disk (encrypted)
3. **Multi-Registry Support**: Manage credentials for multiple registries simultaneously
4. **OAuth2 Device Flow**: For interactive CLI authentication with cloud registries
5. **Certificate-based Authentication**: Support client certificates (mutual TLS)
