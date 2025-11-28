//! HTTP client for OCI registry communication.
//!
//! This module provides a thin HTTP client built on reqwest for interacting
//! with OCI-compliant container registries. It implements the OCI Distribution
//! Specification v2 API.

use crate::digest::Digest;
use crate::error::{Result, RexError};
use reqwest::StatusCode;
use reqwest::blocking::{Client as ReqwestClient, Response};
use serde::Deserialize;
use sha2::{Digest as Sha2Digest, Sha256};
use std::str::FromStr;
use std::time::Duration;

#[cfg(test)]
mod tests;

/// Response from the catalog API endpoint.
#[derive(Debug, Deserialize)]
struct CatalogResponse {
    /// List of repository names
    repositories: Vec<String>,
}

/// Response from the tags list API endpoint.
#[derive(Debug, Deserialize)]
struct TagsResponse {
    /// Repository name
    name: String,
    /// List of tag names
    tags: Vec<String>,
}

/// Version information returned by the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryVersion {
    /// The Docker-Distribution-API-Version header value, if present.
    /// Typically "registry/2.0" for OCI Distribution Spec v2.
    pub api_version: Option<String>,
}

/// Configuration for the HTTP client.
///
/// This struct allows customization of HTTP client behavior such as timeouts
/// and connection pooling. Use the builder pattern to configure:
///
/// # Examples
///
/// ```
/// use librex::client::ClientConfig;
///
/// let config = ClientConfig::new()
///     .with_timeout(60)
///     .with_max_idle_per_host(20);
/// ```
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Request timeout in seconds (default: 30)
    pub timeout_seconds: u64,
    /// Maximum idle connections per host (default: 10)
    pub max_idle_per_host: usize,
    /// Enable Docker Hub compatibility mode (adds "library/" prefix for simple names)
    /// Default: false (works with Zot, GHCR, and most registries)
    pub dockerhub_compat: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_idle_per_host: 10,
            dockerhub_compat: false,
        }
    }
}

impl ClientConfig {
    /// Creates a new configuration with default values.
    ///
    /// Default values:
    /// - timeout: 30 seconds
    /// - max_idle_per_host: 10 connections
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::client::ClientConfig;
    ///
    /// let config = ClientConfig::new();
    /// assert_eq!(config.timeout_seconds, 30);
    /// assert_eq!(config.max_idle_per_host, 10);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the request timeout in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::client::ClientConfig;
    ///
    /// let config = ClientConfig::new().with_timeout(60);
    /// assert_eq!(config.timeout_seconds, 60);
    /// ```
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Sets the maximum idle connections per host.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::client::ClientConfig;
    ///
    /// let config = ClientConfig::new().with_max_idle_per_host(20);
    /// assert_eq!(config.max_idle_per_host, 20);
    /// ```
    pub fn with_max_idle_per_host(mut self, max: usize) -> Self {
        self.max_idle_per_host = max;
        self
    }

    /// Sets the Docker Hub compatibility mode.
    ///
    /// When enabled, the client expects the "library/" prefix for simple repository names.
    /// When disabled (default), it works with registries that don't use this convention.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::client::ClientConfig;
    ///
    /// let config = ClientConfig::new().with_dockerhub_compat(true);
    /// assert_eq!(config.dockerhub_compat, true);
    /// ```
    pub fn with_dockerhub_compat(mut self, enabled: bool) -> Self {
        self.dockerhub_compat = enabled;
        self
    }
}

/// HTTP client for OCI registry operations.
///
/// This client handles all HTTP communication with OCI registries, including
/// connection pooling, timeouts, and TLS configuration.
#[derive(Debug, Clone)]
pub struct Client {
    /// The underlying HTTP client
    http_client: ReqwestClient,
    /// Base registry URL (e.g., "https://registry.example.com")
    registry_url: String,
    /// Optional credentials for authenticated requests
    credentials: Option<crate::auth::Credentials>,
}

impl Client {
    /// Creates a new client for the specified registry URL with default configuration.
    ///
    /// Uses default configuration:
    /// - Timeout: 30 seconds
    /// - Max idle connections per host: 10
    ///
    /// For custom configuration, use [`Client::with_config`].
    ///
    /// # Arguments
    ///
    /// * `registry_url` - The base URL of the OCI registry (e.g., "http://localhost:5000")
    /// * `credentials` - Optional credentials for authenticated requests
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::client::Client;
    ///
    /// // Anonymous access
    /// let client = Client::new("http://localhost:5000", None).unwrap();
    ///
    /// // Authenticated access
    /// use librex::auth::Credentials;
    /// let creds = Credentials::basic("user", "pass");
    /// let client = Client::new("http://localhost:5000", Some(creds)).unwrap();
    /// ```
    pub fn new(registry_url: &str, credentials: Option<crate::auth::Credentials>) -> Result<Self> {
        Self::with_config(registry_url, ClientConfig::default(), credentials)
    }

    /// Creates a new client for the specified registry URL with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `registry_url` - The base URL of the OCI registry (e.g., "http://localhost:5000")
    /// * `config` - Client configuration (timeout, connection pooling, etc.)
    /// * `credentials` - Optional credentials for authenticated requests
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::client::{Client, ClientConfig};
    ///
    /// let config = ClientConfig::new()
    ///     .with_timeout(60)
    ///     .with_max_idle_per_host(20);
    ///
    /// let client = Client::with_config("http://localhost:5000", config, None).unwrap();
    /// ```
    pub fn with_config(
        registry_url: &str,
        config: ClientConfig,
        credentials: Option<crate::auth::Credentials>,
    ) -> Result<Self> {
        // Validate and normalize the registry URL
        let normalized_url = Self::normalize_url(registry_url)?;

        // Build the HTTP client with the provided configuration
        let http_client = ReqwestClient::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .pool_max_idle_per_host(config.max_idle_per_host)
            .build()
            .map_err(|e| RexError::network_with_source("Failed to create HTTP client", e))?;

        Ok(Self {
            http_client,
            registry_url: normalized_url,
            credentials,
        })
    }

    /// Normalizes a registry URL by ensuring it has a scheme and removing trailing slashes.
    fn normalize_url(url: &str) -> Result<String> {
        let url = url.trim();

        // Check if URL is empty
        if url.is_empty() {
            return Err(RexError::validation("Registry URL cannot be empty"));
        }

        // Add default scheme if missing
        let url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("http://{}", url)
        } else {
            url.to_string()
        };

        // Remove trailing slashes
        let url = url.trim_end_matches('/');

        Ok(url.to_string())
    }

    /// Returns the base registry URL.
    pub fn registry_url(&self) -> &str {
        &self.registry_url
    }

    /// Checks if the registry supports the OCI Distribution Specification v2 API.
    ///
    /// This method performs a GET request to the `/v2/` endpoint to verify that
    /// the registry is accessible and supports the OCI Distribution Specification.
    /// It returns version information from the registry's response headers.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::client::Client;
    ///
    /// # fn example() -> librex::error::Result<()> {
    /// let client = Client::new("http://localhost:5000", None)?;
    /// let version = client.check_version()?;
    /// if let Some(api_version) = version.api_version {
    ///     println!("Registry API version: {}", api_version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns `RegistryVersion` containing:
    /// - `api_version`: The Docker-Distribution-API-Version header value (typically "registry/2.0")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The registry is unreachable
    /// - The registry does not support the OCI Distribution Specification
    /// - Authentication is required but not provided
    pub fn check_version(&self) -> Result<RegistryVersion> {
        let url = format!("{}/v2/", self.registry_url);

        let mut request = self.http_client.get(&url);

        // Add Authorization header if credentials are present
        if let Some(ref creds) = self.credentials
            && let Some(auth_header) = creds.to_header_value()
        {
            request = request.header("Authorization", auth_header);
        }

        let response = request
            .send()
            .map_err(|e| Self::translate_reqwest_error(e, &self.registry_url))?;

        // Extract version information from headers before consuming response
        let api_version = response
            .headers()
            .get("Docker-Distribution-API-Version")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self::check_response_status(response)?;

        Ok(RegistryVersion { api_version })
    }

    /// Fetches the catalog of repositories from the registry.
    ///
    /// This method performs a GET request to the `/v2/_catalog` endpoint to retrieve
    /// the list of all repository names in the registry. It automatically handles
    /// pagination and fetches all repositories.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::client::Client;
    ///
    /// # fn example() -> librex::error::Result<()> {
    /// let client = Client::new("http://localhost:5000", None)?;
    /// let repositories = client.fetch_catalog()?;
    /// for repo in repositories {
    ///     println!("{}", repo);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The registry is unreachable
    /// - Authentication is required but not provided
    /// - The response cannot be parsed as valid JSON
    pub fn fetch_catalog(&self) -> Result<Vec<String>> {
        self.fetch_catalog_paginated(None)
    }

    /// Fetches the catalog with optional pagination limit.
    ///
    /// This is the internal implementation that supports pagination. If `limit` is None,
    /// all repositories are fetched by following pagination links. If `limit` is Some(n),
    /// only up to n repositories per page are fetched (useful for testing pagination).
    ///
    /// # Arguments
    ///
    /// * `limit` - Optional maximum number of repositories per page
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::client::Client;
    ///
    /// # fn example() -> librex::error::Result<()> {
    /// let client = Client::new("http://localhost:5000", None)?;
    ///
    /// // Fetch all repositories
    /// let all_repos = client.fetch_catalog_paginated(None)?;
    ///
    /// // Fetch with pagination limit (useful for large registries)
    /// let repos_page = client.fetch_catalog_paginated(Some(100))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn fetch_catalog_paginated(&self, limit: Option<usize>) -> Result<Vec<String>> {
        let mut all_repositories = Vec::new();
        let mut url = format!("{}/v2/_catalog", self.registry_url);

        // Add limit parameter if specified
        if let Some(n) = limit {
            url.push_str(&format!("?n={}", n));
        }

        loop {
            let mut request = self.http_client.get(&url);

            // Add Authorization header if credentials are present
            if let Some(ref creds) = self.credentials
                && let Some(auth_header) = creds.to_header_value()
            {
                request = request.header("Authorization", auth_header);
            }

            let response = request
                .send()
                .map_err(|e| Self::translate_reqwest_error(e, &self.registry_url))?;

            // Extract Link header for pagination before consuming response
            let next_path = Self::extract_next_link(response.headers());

            let response = Self::check_response_status(response)?;

            let catalog: CatalogResponse = response.json().map_err(|e| {
                RexError::validation_with_source("Failed to parse catalog response", e)
            })?;

            all_repositories.extend(catalog.repositories);

            // Check if there's a next page
            if let Some(path) = next_path {
                // Combine registry URL with the path from Link header
                url = format!("{}{}", self.registry_url, path);
            } else {
                break;
            }
        }

        Ok(all_repositories)
    }

    /// Fetches the list of tags for a specific repository.
    ///
    /// This method performs a GET request to the `/v2/<name>/tags/list` endpoint to retrieve
    /// the list of all tag names for a repository. It automatically handles pagination and
    /// fetches all tags.
    ///
    /// # Arguments
    ///
    /// * `repository` - The name of the repository
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::client::Client;
    ///
    /// # fn example() -> librex::error::Result<()> {
    /// let client = Client::new("http://localhost:5000", None)?;
    /// let tags = client.fetch_tags("alpine")?;
    /// for tag in tags {
    ///     println!("{}", tag);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The registry is unreachable
    /// - The repository does not exist
    /// - Authentication is required but not provided
    /// - The response cannot be parsed as valid JSON
    pub fn fetch_tags(&self, repository: &str) -> Result<Vec<String>> {
        self.fetch_tags_paginated(repository, None)
    }

    /// Fetches the list of tags with optional pagination limit.
    ///
    /// This is the internal implementation that supports pagination. If `limit` is None,
    /// all tags are fetched by following pagination links. If `limit` is Some(n),
    /// only up to n tags per page are fetched (useful for testing pagination).
    ///
    /// # Arguments
    ///
    /// * `repository` - The name of the repository
    /// * `limit` - Optional maximum number of tags per page
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::client::Client;
    ///
    /// # fn example() -> librex::error::Result<()> {
    /// let client = Client::new("http://localhost:5000", None)?;
    ///
    /// // Fetch all tags
    /// let all_tags = client.fetch_tags_paginated("alpine", None)?;
    ///
    /// // Fetch with pagination limit
    /// let tags_page = client.fetch_tags_paginated("alpine", Some(100))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn fetch_tags_paginated(
        &self,
        repository: &str,
        limit: Option<usize>,
    ) -> Result<Vec<String>> {
        let mut all_tags = Vec::new();
        let mut url = format!("{}/v2/{}/tags/list", self.registry_url, repository);

        // Add limit parameter if specified
        if let Some(n) = limit {
            url.push_str(&format!("?n={}", n));
        }

        loop {
            let mut request = self.http_client.get(&url);

            // Add Authorization header if credentials are present
            if let Some(ref creds) = self.credentials
                && let Some(auth_header) = creds.to_header_value()
            {
                request = request.header("Authorization", auth_header);
            }

            let response = request
                .send()
                .map_err(|e| Self::translate_reqwest_error(e, &self.registry_url))?;

            // Extract Link header for pagination before consuming response
            let next_path = Self::extract_next_link(response.headers());

            let response = Self::check_response_status(response)?;

            let tags_response: TagsResponse = response.json().map_err(|e| {
                RexError::validation_with_source("Failed to parse tags response", e)
            })?;

            // Validate that the response is for the correct repository
            if tags_response.name != repository {
                return Err(RexError::validation(format!(
                    "Registry returned tags for '{}' but expected '{}'",
                    tags_response.name, repository
                )));
            }

            all_tags.extend(tags_response.tags);

            // Check if there's a next page
            if let Some(path) = next_path {
                // Combine registry URL with the path from Link header
                url = format!("{}{}", self.registry_url, path);
            } else {
                break;
            }
        }

        Ok(all_tags)
    }

    /// Fetches a manifest for a specific image reference.
    ///
    /// This method performs a GET request to the `/v2/<name>/manifests/<reference>` endpoint
    /// to retrieve the manifest for an image. The reference can be either a tag name or a digest.
    ///
    /// # Arguments
    ///
    /// * `repository` - The name of the repository
    /// * `reference` - The tag name (e.g., "latest") or digest (e.g., "sha256:abc123...")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::client::Client;
    ///
    /// # fn example() -> librex::error::Result<()> {
    /// let client = Client::new("http://localhost:5000", None)?;
    ///
    /// // Fetch by tag
    /// let (manifest_bytes, digest) = client.fetch_manifest("alpine", "latest")?;
    ///
    /// // Fetch by digest
    /// let (manifest_bytes, digest) = client.fetch_manifest(
    ///     "alpine",
    ///     "sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b"
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(Vec<u8>, String)` where:
    /// - The first element is the raw manifest bytes
    /// - The second element is the manifest digest (from header or computed)
    ///
    /// If the Docker-Content-Digest header is missing, the digest is computed
    /// from the response bytes using SHA256.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The registry is unreachable
    /// - The repository or reference does not exist
    /// - Authentication is required but not provided
    pub fn fetch_manifest(&self, repository: &str, reference: &str) -> Result<(Vec<u8>, String)> {
        let url = format!(
            "{}/v2/{}/manifests/{}",
            self.registry_url, repository, reference
        );

        let mut request = self
            .http_client
            .get(&url)
            // Add Accept headers for OCI and Docker manifest types
            .header(
                "Accept",
                "application/vnd.oci.image.manifest.v1+json, \
                 application/vnd.oci.image.index.v1+json, \
                 application/vnd.docker.distribution.manifest.v2+json, \
                 application/vnd.docker.distribution.manifest.list.v2+json",
            );

        // Add Authorization header if credentials are present
        if let Some(ref creds) = self.credentials
            && let Some(auth_header) = creds.to_header_value()
        {
            request = request.header("Authorization", auth_header);
        }

        let response = request
            .send()
            .map_err(|e| Self::translate_reqwest_error(e, &self.registry_url))?;

        // Extract Docker-Content-Digest header before consuming response
        let digest_from_header = response
            .headers()
            .get("Docker-Content-Digest")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let response = Self::check_response_status(response)?;

        // Get raw bytes for the manifest
        let manifest_bytes = response
            .bytes()
            .map_err(|e| RexError::network_with_source("Failed to read manifest response", e))?;

        // Use header digest if available, otherwise compute from bytes
        let digest = if let Some(d) = digest_from_header {
            d
        } else {
            // Compute digest from bytes using sha256
            use sha2::{Digest as Sha2Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&manifest_bytes);
            format!("sha256:{:x}", hasher.finalize())
        };

        Ok((manifest_bytes.to_vec(), digest))
    }

    /// Fetches a blob (layer or config) from the registry.
    ///
    /// This method performs a GET request to the `/v2/<name>/blobs/<digest>` endpoint
    /// to retrieve a blob. The blob content is verified against the provided digest.
    /// Redirects (e.g., to CDN or storage backends) are handled automatically by reqwest.
    ///
    /// # Arguments
    ///
    /// * `repository` - The name of the repository
    /// * `digest` - The content digest of the blob (e.g., "sha256:abc123...")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::client::Client;
    /// use std::str::FromStr;
    ///
    /// # fn example() -> librex::error::Result<()> {
    /// let client = Client::new("http://localhost:5000", None)?;
    ///
    /// // Fetch a blob by digest
    /// let blob_data = client.fetch_blob(
    ///     "alpine",
    ///     "sha256:4abcf20661432fb2d719b4568d94db3b6cf9b44bf2a3e1c2c6d0c89fd9e6e0b2"
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns the raw blob content as `Vec<u8>`. The content is guaranteed to match
    /// the provided digest.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The registry is unreachable
    /// - The blob does not exist
    /// - Authentication is required but not provided
    /// - The downloaded content does not match the expected digest
    /// - The digest format is invalid
    pub fn fetch_blob(&self, repository: &str, digest: &str) -> Result<Vec<u8>> {
        // Parse and validate the digest format
        let expected_digest = Digest::from_str(digest)?;

        let url = format!("{}/v2/{}/blobs/{}", self.registry_url, repository, digest);

        let mut request = self.http_client.get(&url);

        // Add Authorization header if credentials are present
        if let Some(ref creds) = self.credentials
            && let Some(auth_header) = creds.to_header_value()
        {
            request = request.header("Authorization", auth_header);
        }

        let response = request
            .send()
            .map_err(|e| Self::translate_reqwest_error(e, &self.registry_url))?;

        let response = Self::check_response_status(response)?;

        // Download the blob content
        let blob_bytes = response
            .bytes()
            .map_err(|e| RexError::network_with_source("Failed to read blob response", e))?;

        // Verify the digest matches what we downloaded
        // Currently only supporting SHA256
        if expected_digest.algorithm() != "sha256" {
            return Err(RexError::validation(format!(
                "Unsupported digest algorithm: {}. Only sha256 is currently supported",
                expected_digest.algorithm()
            )));
        }

        let mut hasher = Sha256::new();
        hasher.update(&blob_bytes);
        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash != expected_digest.hex() {
            return Err(RexError::validation(format!(
                "Blob digest mismatch: expected {}, computed sha256:{}",
                digest, computed_hash
            )));
        }

        Ok(blob_bytes.to_vec())
    }

    /// Deletes a manifest from the registry by digest.
    ///
    /// This method performs a DELETE request to the `/v2/<name>/manifests/<digest>` endpoint.
    /// According to the OCI Distribution Specification, manifests can only be deleted by digest,
    /// not by tag. To delete a tag, you must first resolve it to a digest using `fetch_manifest`.
    ///
    /// **Important**: Not all registries support manifest deletion. The registry must be configured
    /// to allow DELETE operations. If deletion is not enabled, this method will return an error
    /// with status 405 (Method Not Allowed).
    ///
    /// # Arguments
    ///
    /// * `repository` - The name of the repository
    /// * `digest` - The manifest digest to delete (must be a digest, not a tag)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::client::Client;
    ///
    /// # fn example() -> librex::error::Result<()> {
    /// let client = Client::new("http://localhost:5000", None)?;
    ///
    /// // First, get the digest for a tag
    /// let (_, digest) = client.fetch_manifest("alpine", "latest")?;
    ///
    /// // Then delete using the digest
    /// client.delete_manifest("alpine", &digest)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the manifest was successfully deleted (HTTP 202 or 204).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The registry is unreachable
    /// - The manifest does not exist (404)
    /// - Authentication is required but not provided (401)
    /// - The operation is forbidden (403)
    /// - Deletion is not enabled on the registry (405)
    /// - The digest format is invalid
    pub fn delete_manifest(&self, repository: &str, digest: &str) -> Result<()> {
        let url = format!(
            "{}/v2/{}/manifests/{}",
            self.registry_url, repository, digest
        );

        let mut request = self.http_client.delete(&url);

        // Add Authorization header if credentials are present
        if let Some(ref creds) = self.credentials
            && let Some(auth_header) = creds.to_header_value()
        {
            request = request.header("Authorization", auth_header);
        }

        let response = request
            .send()
            .map_err(|e| Self::translate_reqwest_error(e, &self.registry_url))?;

        let status = response.status();

        // Handle successful deletion
        if status == StatusCode::ACCEPTED || status == StatusCode::NO_CONTENT {
            return Ok(());
        }

        // Handle error cases
        let url_str = response.url().to_string();
        let error_body = response
            .text()
            .unwrap_or_else(|_| String::from("(unable to read response body)"));

        match status {
            StatusCode::NOT_FOUND => Err(RexError::not_found(
                "manifest",
                &format!("{} in {}", digest, repository),
            )),
            StatusCode::UNAUTHORIZED => Err(RexError::authentication(
                format!("Authentication required to delete manifest: {}", error_body),
                Some(401),
            )),
            StatusCode::FORBIDDEN => Err(RexError::authentication(
                format!("Permission denied to delete manifest: {}", error_body),
                Some(403),
            )),
            StatusCode::METHOD_NOT_ALLOWED => Err(RexError::validation(
                "Manifest deletion is not enabled on this registry. \
                 Check registry configuration (e.g., Zot requires 'storage.gc: true' and \
                 'http.allowDelete: true')"
                    .to_string(),
            )),
            StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT => Err(RexError::server(
                format!("Server error while deleting manifest: {}", error_body),
                status.as_u16(),
            )),
            _ => Err(RexError::network(format!(
                "Failed to delete manifest: HTTP {} from {}: {}",
                status.as_u16(),
                url_str,
                error_body
            ))),
        }
    }

    /// Extracts the next page URL from the Link header.
    ///
    /// The OCI Distribution Specification uses the Link header for pagination:
    /// `Link: </v2/_catalog?n=100&last=repo99>; rel="next"`
    fn extract_next_link(headers: &reqwest::header::HeaderMap) -> Option<String> {
        let link_header = headers.get(reqwest::header::LINK)?;
        let link_str = link_header.to_str().ok()?;

        // Parse the Link header to find rel="next"
        // Format: </v2/_catalog?n=100&last=repo99>; rel="next"
        for link_part in link_str.split(',') {
            let link_part = link_part.trim();

            // Check if this is the "next" relation
            if link_part.contains("rel=\"next\"") || link_part.contains("rel='next'") {
                // Extract URL between < and >
                if let Some(start) = link_part.find('<')
                    && let Some(end) = link_part.find('>')
                {
                    let path = &link_part[start + 1..end];
                    // The path is relative, so we need to combine it with the registry URL
                    // Since the path already starts with /v2/, we can just append it
                    return Some(path.to_string());
                }
            }
        }

        None
    }

    /// Parses the Retry-After header from a rate limit response.
    ///
    /// The Retry-After header can be in two formats per RFC 7231:
    /// 1. Delay-seconds: `Retry-After: 120`
    /// 2. HTTP-date: `Retry-After: Wed, 21 Oct 2025 07:28:00 GMT`
    ///
    /// Returns the retry delay in seconds, or None if the header is missing or invalid.
    fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<u64> {
        let retry_after = headers.get(reqwest::header::RETRY_AFTER)?;
        let retry_str = retry_after.to_str().ok()?;

        // Try parsing as delay-seconds (integer)
        if let Ok(seconds) = retry_str.parse::<u64>() {
            return Some(seconds);
        }

        // Try parsing as HTTP-date
        if let Ok(datetime) = httpdate::parse_http_date(retry_str) {
            // Calculate seconds from now until the retry time
            let now = std::time::SystemTime::now();

            // If datetime is in the future, calculate how many seconds until then
            if datetime > now {
                if let Ok(duration) = datetime.duration_since(now) {
                    return Some(duration.as_secs());
                }
            } else {
                // If datetime is in the past, return 0 (can retry immediately)
                return Some(0);
            }
        }

        // Invalid or unparseable format
        None
    }

    /// Translates a reqwest error into a RexError.
    fn translate_reqwest_error(error: reqwest::Error, registry_url: &str) -> RexError {
        if error.is_timeout() {
            RexError::network(format!(
                "Request to {} timed out after 30 seconds",
                registry_url
            ))
        } else if error.is_connect() {
            RexError::network_with_source(
                format!("Failed to connect to registry at {}", registry_url),
                error,
            )
        } else if error.is_request() {
            RexError::network_with_source(
                format!("Failed to send request to {}", registry_url),
                error,
            )
        } else {
            RexError::network_with_source(
                format!("Network error communicating with {}", registry_url),
                error,
            )
        }
    }

    /// Checks the HTTP response status and translates errors to RexError.
    fn check_response_status(response: Response) -> Result<Response> {
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

        // Extract headers and URL before consuming the response
        let headers = response.headers().clone();
        let url = response.url().to_string();

        // Try to extract error message from response body
        let error_body = response
            .text()
            .unwrap_or_else(|_| String::from("(unable to read response body)"));

        match status {
            StatusCode::UNAUTHORIZED => Err(RexError::authentication(
                format!("Authentication required for {}: {}", url, error_body),
                Some(401),
            )),
            StatusCode::FORBIDDEN => Err(RexError::authentication(
                format!("Access forbidden for {}: {}", url, error_body),
                Some(403),
            )),
            StatusCode::NOT_FOUND => Err(RexError::not_found("endpoint", &url)),
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = Self::parse_retry_after(&headers);
                Err(RexError::rate_limit(
                    format!("Rate limit exceeded for {}", url),
                    retry_after,
                ))
            }
            StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT => Err(RexError::server(
                format!("Server error from {}: {}", url, error_body),
                status.as_u16(),
            )),
            _ => Err(RexError::network(format!(
                "HTTP {} from {}: {}",
                status.as_u16(),
                url,
                error_body
            ))),
        }
    }
}
