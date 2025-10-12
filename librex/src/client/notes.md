# Client Module - Implementation Notes

## Overview

The client module provides a thin HTTP client wrapper around `reqwest` for communicating with OCI-compliant container registries. It implements the OCI Distribution Specification v2 API.

## Architecture Decision

We chose to build our own thin HTTP client on top of `reqwest` instead of using the `oci-client` crate for the following reasons:

1. **Type Consistency**: We use `oci-spec` types throughout the codebase. The `oci-client` crate defines its own types (`OciImageManifest`, `OciDescriptor`, etc.) that are not compatible with `oci-spec` types, which would require conversion layers.

2. **Control**: Building our own client gives us full control over:
   - Error handling and translation to our `RexError` types
   - Caching integration
   - Retry logic
   - Request/response logging
   - Custom headers

3. **Simplicity**: The OCI Distribution Specification is straightforward, and implementing it directly is simpler than managing type conversions between `oci-client` and `oci-spec`.

4. **Learning**: Implementing the protocol ourselves provides deep understanding for debugging and troubleshooting.

## Implementation Strategy

We're following an incremental TDD approach:

### Phase 1: Basic Client ✓
- Client struct with HTTP client wrapper
- URL normalization
- Basic configuration (timeout, connection pooling)

### Phase 2: Version Check (Next)
- Implement `GET /v2/` endpoint
- Verify registry supports OCI Distribution Specification
- Error handling for connection issues

### Phase 3: Catalog Operations
- Implement `GET /v2/_catalog` endpoint
- Handle pagination
- Parse JSON responses

### Phase 4: Tag Operations
- Implement `GET /v2/<name>/tags/list` endpoint
- Handle pagination

### Phase 5: Manifest Operations
- Implement `GET /v2/<name>/manifests/<reference>` endpoint
- Content negotiation (Accept headers)
- Return `oci_spec::image::ImageManifest`

### Phase 6: Authentication
- Handle WWW-Authenticate challenges
- Bearer token authentication
- Basic authentication
- Token caching

### Phase 7: Blob Operations
- Implement `GET /v2/<name>/blobs/<digest>` endpoint
- Handle redirects
- Stream large blobs

## Dependencies

- `reqwest` - HTTP client library with async support
  - Features: `json`, `rustls-tls`
  - Provides connection pooling, timeouts, TLS support
- `tokio` - Async runtime
  - Features: `rt`, `macros`
  - Required by reqwest for async operations

## Configuration

The client is configured with:
- **Timeout**: 30 seconds (default)
- **Connection pooling**: 10 connections per host
- **TLS**: Enabled by default using rustls

## Error Handling

All HTTP errors are translated to our `RexError` types:
- Network errors → `RexError::Network`
- 401/403 → `RexError::Authentication`
- 404 → `RexError::NotFound`
- 429 → `RexError::RateLimit`
- 500/503 → `RexError::Server`
- Invalid responses → `RexError::Validation`

## Testing Strategy

- Unit tests for URL normalization and client creation
- Integration tests will be added later when we have more complex interactions
- Tests use the local test registry when available
- Mock responses for unit tests of HTTP operations

## Future Enhancements

- Custom CA certificates for private registries
- Configurable retries with exponential backoff
- Request/response logging with `tracing`
- Proxy support (via reqwest's built-in support)
- HTTP/2 support
- Connection reuse metrics
