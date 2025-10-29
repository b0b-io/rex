//! Registry operations module.
//!
//! This module provides high-level operations for interacting with OCI registries,
//! including listing repositories, fetching tags, retrieving manifests and blobs.
//! It orchestrates the client, authentication, and cache modules to provide a
//! seamless API for registry interactions.

use crate::auth::Credentials;
use crate::cache::{Cache, CacheType};
use crate::client::Client;
use crate::digest::Digest;
use crate::error::Result;
use crate::oci::ManifestOrIndex;
use crate::reference::Reference;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Response from the catalog endpoint listing repositories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct CatalogResponse {
    /// List of repository names.
    pub repositories: Vec<String>,
}

/// Response from the tags endpoint listing tags for a repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct TagsResponse {
    /// Repository name.
    pub name: String,
    /// List of tags.
    pub tags: Vec<String>,
}

/// High-level registry client that orchestrates HTTP client, auth, and cache.
pub struct Registry {
    /// HTTP client for registry communication.
    client: Client,
    /// Optional cache for registry responses.
    cache: Option<Cache>,
    /// Optional credentials for authentication.
    credentials: Option<Credentials>,
}

impl Registry {
    /// Creates a new `Registry` instance.
    ///
    /// # Arguments
    ///
    /// * `client` - HTTP client configured for the registry
    /// * `cache` - Optional cache for performance optimization
    /// * `credentials` - Optional credentials for authentication
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::client::Client;
    /// use librex::registry::Registry;
    ///
    /// let client = Client::new("http://localhost:5000", None).unwrap();
    /// let registry = Registry::new(client, None, None);
    /// ```
    pub fn new(client: Client, cache: Option<Cache>, credentials: Option<Credentials>) -> Self {
        Self {
            client,
            cache,
            credentials,
        }
    }

    /// Lists all repositories in the registry (catalog operation).
    ///
    /// This method fetches the repository catalog from the registry. It will use
    /// the cache if available and not expired, otherwise it fetches from the registry.
    ///
    /// # Returns
    ///
    /// A vector of repository names available in the registry.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librex::client::Client;
    /// # use librex::registry::Registry;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:5000", None)?;
    /// let mut registry = Registry::new(client, None, None);
    ///
    /// let repos = registry.list_repositories().await?;
    /// for repo in repos {
    ///     println!("{}", repo);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_repositories(&mut self) -> Result<Vec<String>> {
        let cache_key = "catalog";

        // Try cache first
        if let Some(cache) = &mut self.cache
            && let Some(cached) = cache.get::<CatalogResponse>(cache_key)?
        {
            return Ok(cached.repositories);
        }

        // Fetch from registry - client returns Vec<String> directly
        let repositories = self.client.fetch_catalog().await?;

        // Cache the result
        if let Some(cache) = &mut self.cache {
            let catalog = CatalogResponse {
                repositories: repositories.clone(),
            };
            cache.set(cache_key, &catalog, CacheType::Catalog)?;
        }

        Ok(repositories)
    }

    /// Lists all tags for a specific repository.
    ///
    /// # Arguments
    ///
    /// * `repository` - The name of the repository (e.g., "alpine", "library/ubuntu")
    ///
    /// # Returns
    ///
    /// A vector of tag names available for the repository.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librex::client::Client;
    /// # use librex::registry::Registry;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:5000", None)?;
    /// let mut registry = Registry::new(client, None, None);
    ///
    /// let tags = registry.list_tags("alpine").await?;
    /// for tag in tags {
    ///     println!("{}", tag);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_tags(&mut self, repository: &str) -> Result<Vec<String>> {
        let cache_key = format!("{}/_tags", repository);

        // Try cache first
        if let Some(cache) = &mut self.cache
            && let Some(cached) = cache.get::<TagsResponse>(&cache_key)?
        {
            return Ok(cached.tags);
        }

        // Fetch from registry - client returns Vec<String> directly
        let tags = self.client.fetch_tags(repository).await?;

        // Cache the result
        if let Some(cache) = &mut self.cache {
            let tags_response = TagsResponse {
                name: repository.to_string(),
                tags: tags.clone(),
            };
            cache.set(&cache_key, &tags_response, CacheType::Tags)?;
        }

        Ok(tags)
    }

    /// Retrieves a manifest or index for a specific image reference.
    ///
    /// This method automatically detects whether the image is a single-platform
    /// manifest or a multi-platform index and returns the appropriate type.
    ///
    /// # Arguments
    ///
    /// * `reference` - The image reference (repository:tag or repository@digest)
    ///
    /// # Returns
    ///
    /// A `ManifestOrIndex` enum containing either:
    /// - `Manifest` for single-platform images
    /// - `Index` for multi-platform images
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librex::client::Client;
    /// # use librex::reference::Reference;
    /// # use librex::registry::Registry;
    /// # use std::str::FromStr;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:5000", None)?;
    /// let mut registry = Registry::new(client, None, None);
    /// let reference = Reference::from_str("alpine:latest")?;
    ///
    /// let manifest_or_index = registry.get_manifest(&reference).await?;
    /// match manifest_or_index {
    ///     librex::ManifestOrIndex::Manifest(manifest) => {
    ///         println!("Single-platform image with {} layers", manifest.layers().len());
    ///     }
    ///     librex::ManifestOrIndex::Index(index) => {
    ///         println!("Multi-platform image with {} platforms", index.manifests().len());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_manifest(&mut self, reference: &Reference) -> Result<ManifestOrIndex> {
        // For digest references, we can cache by digest
        let cache_key = if let Some(digest) = reference.digest() {
            format!("{}/manifests/{}", reference.repository(), digest)
        } else {
            // Tag-based references change over time, so we use a different cache key
            format!(
                "{}/tags/{}/manifest",
                reference.repository(),
                reference.tag().unwrap_or("latest")
            )
        };

        // Try cache first - cache stores raw bytes which we parse
        // Note: Both digest and tag references are cached, but with different TTLs
        // - Digest-based: Long TTL (immutable content)
        // - Tag-based: Shorter TTL via CacheType::Manifest (content can change)
        if let Some(cache) = &mut self.cache
            && let Some(cached_bytes) = cache.get::<Vec<u8>>(&cache_key)?
        {
            return ManifestOrIndex::from_bytes(&cached_bytes);
        }

        // Fetch from registry - client returns (Vec<u8>, String) tuple
        let reference_str = if let Some(digest) = reference.digest() {
            digest
        } else {
            reference.tag().unwrap_or("latest")
        };

        let (manifest_bytes, _digest) = self
            .client
            .fetch_manifest(reference.repository(), reference_str)
            .await?;

        // Parse the manifest or index
        let manifest_or_index = ManifestOrIndex::from_bytes(&manifest_bytes)?;

        // Cache the raw bytes (not the parsed struct)
        if let Some(cache) = &mut self.cache {
            cache.set(&cache_key, &manifest_bytes, CacheType::Manifest)?;
        }

        Ok(manifest_or_index)
    }

    /// Retrieves a blob (layer or config) by digest.
    ///
    /// Blobs are immutable and content-addressed by digest. They are cached
    /// globally (independent of repository/registry) since the same digest
    /// always represents the same content.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository name
    /// * `digest` - The content digest of the blob
    ///
    /// # Returns
    ///
    /// The raw blob content as bytes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librex::client::Client;
    /// # use librex::digest::Digest;
    /// # use librex::registry::Registry;
    /// # use std::str::FromStr;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:5000", None)?;
    /// let mut registry = Registry::new(client, None, None);
    /// let digest = Digest::from_str("sha256:abc123...")?;
    ///
    /// let blob = registry.get_blob("alpine", &digest).await?;
    /// println!("Blob size: {} bytes", blob.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_blob(&mut self, repository: &str, digest: &Digest) -> Result<Vec<u8>> {
        // Cache key is global (not repository-specific) since blobs are content-addressed
        let cache_key = format!("blobs/{}", digest);

        // Try cache first
        if let Some(cache) = &mut self.cache
            && let Some(cached_bytes) = cache.get::<Vec<u8>>(&cache_key)?
        {
            return Ok(cached_bytes);
        }

        // Fetch from registry
        let blob_bytes = self
            .client
            .fetch_blob(repository, &digest.to_string())
            .await?;

        // Cache the blob with Config type (very long TTL for immutable content)
        if let Some(cache) = &mut self.cache {
            cache.set(&cache_key, &blob_bytes, crate::cache::CacheType::Config)?;
        }

        Ok(blob_bytes)
    }

    /// Checks if the registry is accessible and supports the OCI Distribution Specification.
    ///
    /// This performs a version check by calling the `/v2/` endpoint.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the registry is accessible, error otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use librex::client::Client;
    /// # use librex::registry::Registry;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:5000", None)?;
    /// let mut registry = Registry::new(client, None, None);
    ///
    /// match registry.check_version().await {
    ///     Ok(_) => println!("Registry is accessible"),
    ///     Err(e) => println!("Failed to connect: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check_version(&mut self) -> Result<()> {
        self.client.check_version().await.map(|_| ())
    }

    /// Sets the credentials for authenticated requests.
    ///
    /// # Arguments
    ///
    /// * `credentials` - The credentials to use for authentication
    pub fn set_credentials(&mut self, credentials: Credentials) {
        self.credentials = Some(credentials);
    }

    /// Returns a reference to the current credentials, if any.
    pub fn credentials(&self) -> Option<&Credentials> {
        self.credentials.as_ref()
    }

    /// Clears the current credentials, switching to anonymous access.
    pub fn clear_credentials(&mut self) {
        self.credentials = None;
    }
}
