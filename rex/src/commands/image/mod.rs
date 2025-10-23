use crate::config;
use crate::format::Formattable;
use librex::auth::CredentialStore;
use serde::Serialize;
use std::str::FromStr;
use tabled::Tabled;

pub mod handlers;

/// Image information for listing
#[derive(Debug, Serialize, Tabled)]
pub struct ImageInfo {
    /// Image name (repository)
    #[tabled(rename = "NAME")]
    pub name: String,
    /// Number of tags
    #[tabled(rename = "TAGS")]
    pub tags: usize,
    /// Last updated timestamp
    #[tabled(rename = "LAST UPDATED")]
    pub last_updated: String,
}

impl ImageInfo {
    /// Create a new ImageInfo
    pub fn new(name: String, tags: usize, last_updated: Option<String>) -> Self {
        Self {
            name,
            tags,
            last_updated: last_updated.unwrap_or_else(|| "N/A".to_string()),
        }
    }
}

impl Formattable for ImageInfo {
    fn format_pretty(&self) -> String {
        format!("{:20} {:5} {}", self.name, self.tags, self.last_updated)
    }
}

/// Tag information for a specific image
#[derive(Debug, Serialize, Tabled)]
pub struct TagInfo {
    /// Tag name
    #[tabled(rename = "TAG")]
    pub tag: String,
}

impl TagInfo {
    /// Create a new TagInfo
    pub fn new(tag: String) -> Self {
        Self { tag }
    }
}

impl Formattable for TagInfo {
    fn format_pretty(&self) -> String {
        self.tag.clone()
    }
}

/// Detailed information about a specific image tag
#[derive(Debug, Serialize)]
pub struct ImageDetails {
    /// Full reference (name:tag or name@digest)
    pub reference: String,
    /// Manifest digest
    pub digest: String,
    /// Manifest type (OCI Image Index, OCI Image Manifest, etc.)
    pub manifest_type: String,
    /// Total size in bytes
    pub size: u64,
    /// List of platforms (for multi-platform images)
    pub platforms: Vec<String>,
    /// Number of layers
    pub layers: usize,
}

impl ImageDetails {
    /// Create a new ImageDetails
    pub fn new(
        reference: String,
        digest: String,
        manifest_type: String,
        size: u64,
        platforms: Vec<String>,
        layers: usize,
    ) -> Self {
        Self {
            reference,
            digest,
            manifest_type,
            size,
            platforms,
            layers,
        }
    }
}

impl Formattable for ImageDetails {
    fn format_pretty(&self) -> String {
        fn format_bytes(bytes: u64) -> String {
            const KB: u64 = 1024;
            const MB: u64 = KB * 1024;
            const GB: u64 = MB * 1024;

            if bytes >= GB {
                format!("{:.2} GB", bytes as f64 / GB as f64)
            } else if bytes >= MB {
                format!("{:.2} MB", bytes as f64 / MB as f64)
            } else if bytes >= KB {
                format!("{:.2} KB", bytes as f64 / KB as f64)
            } else {
                format!("{} B", bytes)
            }
        }

        let mut output = String::new();
        output.push_str(&format!("Image: {}\n", self.reference));
        output.push_str(&format!("Digest: {}\n", self.digest));
        output.push_str(&format!("Type: {}\n", self.manifest_type));
        output.push_str(&format!("Size: {}\n", format_bytes(self.size)));

        if !self.platforms.is_empty() {
            output.push_str("\nPlatforms:\n");
            for platform in &self.platforms {
                output.push_str(&format!("  {}\n", platform));
            }
        }

        output.push_str(&format!("\nLayers: {}\n", self.layers));

        output
    }
}

/// List all repositories (images) in a registry
///
/// # Arguments
///
/// * `registry_url` - URL of the registry to query
/// * `filter` - Optional filter pattern for fuzzy matching
/// * `limit` - Optional limit on number of results
///
/// # Returns
///
/// Returns a vector of ImageInfo structs with repository information
pub(crate) async fn list_images(
    registry_url: &str,
    filter: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<ImageInfo>, String> {
    // Get cache directory from config (per-registry subdirectory)
    let cache_dir = get_registry_cache_dir(registry_url)?;

    // Load credentials if available
    let creds_path = config::get_credentials_path();
    let credentials = if creds_path.exists() {
        if let Ok(store) = librex::auth::FileCredentialStore::new(creds_path) {
            store.get(registry_url).ok().flatten()
        } else {
            None
        }
    } else {
        None
    };

    // Build Rex instance with cache and credentials
    let mut builder = librex::Rex::builder()
        .registry_url(registry_url)
        .with_cache(cache_dir);

    if let Some(creds) = credentials {
        builder = builder.with_credentials(creds);
    }

    let mut rex = builder
        .build()
        .await
        .map_err(|e| format!("Failed to connect to registry: {}", e))?;

    // List repositories
    let repos = if let Some(pattern) = filter {
        // Use fuzzy search if filter is provided
        rex.search_repositories(pattern)
            .await
            .map_err(|e| format!("Failed to search repositories: {}", e))?
            .into_iter()
            .map(|r| r.value)
            .collect()
    } else {
        // List all repositories
        rex.list_repositories()
            .await
            .map_err(|e| format!("Failed to list repositories: {}", e))?
    };

    // Apply limit if specified
    let repos: Vec<String> = if let Some(n) = limit {
        repos.into_iter().take(n).collect()
    } else {
        repos
    };

    // For each repository, get tag count
    // TODO: In the future, we could also fetch last updated time from manifest metadata
    let mut images = Vec::new();
    for repo in repos {
        let tags = rex
            .list_tags(&repo)
            .await
            .map_err(|e| format!("Failed to list tags for {}: {}", repo, e))?;

        images.push(ImageInfo::new(repo, tags.len(), None));
    }

    Ok(images)
}

/// List all tags for a specific image (repository)
///
/// # Arguments
///
/// * `registry_url` - URL of the registry to query
/// * `image_name` - Name of the repository/image
/// * `filter` - Optional filter pattern for fuzzy matching
/// * `limit` - Optional limit on number of results
///
/// # Returns
///
/// Returns a vector of TagInfo structs with tag information
pub(crate) async fn list_tags(
    registry_url: &str,
    image_name: &str,
    filter: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<TagInfo>, String> {
    // Get cache directory from config (per-registry subdirectory)
    let cache_dir = get_registry_cache_dir(registry_url)?;

    // Load credentials if available
    let creds_path = config::get_credentials_path();
    let credentials = if creds_path.exists() {
        if let Ok(store) = librex::auth::FileCredentialStore::new(creds_path) {
            store.get(registry_url).ok().flatten()
        } else {
            None
        }
    } else {
        None
    };

    // Build Rex instance with cache and credentials
    let mut builder = librex::Rex::builder()
        .registry_url(registry_url)
        .with_cache(cache_dir);

    if let Some(creds) = credentials {
        builder = builder.with_credentials(creds);
    }

    let mut rex = builder
        .build()
        .await
        .map_err(|e| format!("Failed to connect to registry: {}", e))?;

    // List tags for the repository
    let tags = if let Some(pattern) = filter {
        // Use fuzzy search if filter is provided
        rex.search_tags(image_name, pattern)
            .await
            .map_err(|e| format!("Failed to search tags: {}", e))?
            .into_iter()
            .map(|r| r.value)
            .collect()
    } else {
        // List all tags
        rex.list_tags(image_name)
            .await
            .map_err(|e| format!("Failed to list tags: {}", e))?
    };

    // Apply limit if specified
    let tags: Vec<String> = if let Some(n) = limit {
        tags.into_iter().take(n).collect()
    } else {
        tags
    };

    // Convert to TagInfo
    let tag_infos = tags.into_iter().map(TagInfo::new).collect();

    Ok(tag_infos)
}

/// Get detailed information for a specific image reference (name:tag or name@digest)
///
/// # Arguments
///
/// * `registry_url` - URL of the registry to query
/// * `reference_str` - Full image reference (e.g., "alpine:latest" or "alpine@sha256:...")
///
/// # Returns
///
/// Returns ImageDetails with manifest information
pub(crate) async fn get_image_details(
    registry_url: &str,
    reference_str: &str,
) -> Result<ImageDetails, String> {
    // Get cache directory from config (per-registry subdirectory)
    let cache_dir = get_registry_cache_dir(registry_url)?;

    // Load credentials if available
    let creds_path = config::get_credentials_path();
    let credentials = if creds_path.exists() {
        if let Ok(store) = librex::auth::FileCredentialStore::new(creds_path) {
            store.get(registry_url).ok().flatten()
        } else {
            None
        }
    } else {
        None
    };

    // Build Rex instance with cache and credentials
    let mut builder = librex::Rex::builder()
        .registry_url(registry_url)
        .with_cache(cache_dir);

    if let Some(creds) = credentials {
        builder = builder.with_credentials(creds);
    }

    let mut rex = builder
        .build()
        .await
        .map_err(|e| format!("Failed to connect to registry: {}", e))?;

    // Parse the reference to validate it
    let reference = librex::reference::Reference::from_str(reference_str)
        .map_err(|e| format!("Invalid image reference: {}", e))?;

    // Get the manifest (Rex::get_manifest expects a string reference)
    let manifest_or_index = rex
        .get_manifest(reference_str)
        .await
        .map_err(|e| format!("Failed to fetch manifest: {}", e))?;

    // Extract details based on manifest type
    let (manifest_type, size, platforms, layers) = match manifest_or_index {
        librex::oci::ManifestOrIndex::Manifest(manifest) => {
            // Single-platform image
            let total_size: u64 = manifest.layers().iter().map(|layer| layer.size()).sum();
            let layer_count = manifest.layers().len();

            // Get platform from config blob
            let platform = {
                // Parse config digest
                let config_digest_str = manifest.config().digest().to_string();
                let config_digest = librex::digest::Digest::from_str(&config_digest_str)
                    .map_err(|e| format!("Invalid config digest: {}", e))?;

                // Fetch config blob
                let config_bytes = rex
                    .get_blob(reference.repository(), &config_digest)
                    .await
                    .map_err(|e| format!("Failed to fetch config blob: {}", e))?;

                // Parse config JSON
                let config: librex::oci::ImageConfiguration = serde_json::from_slice(&config_bytes)
                    .map_err(|e| format!("Failed to parse config: {}", e))?;

                // Extract platform info
                vec![format!("{}/{}", config.os(), config.architecture())]
            };

            (
                "OCI Image Manifest".to_string(),
                total_size,
                platform,
                layer_count,
            )
        }
        librex::oci::ManifestOrIndex::Index(index) => {
            // Multi-platform image
            let platforms: Vec<String> = index
                .manifests()
                .iter()
                .filter_map(|desc| {
                    desc.platform()
                        .as_ref()
                        .map(|p| format!("{}/{}", p.os(), p.architecture()))
                })
                .collect();

            // Sum up sizes of all platform manifests
            let total_size: u64 = index.manifests().iter().map(|desc| desc.size()).sum();

            let layer_count = index.manifests().len();

            (
                "OCI Image Index (multi-platform)".to_string(),
                total_size,
                platforms,
                layer_count,
            )
        }
    };

    // Get the digest - use the one from reference if available, otherwise we'd need to compute it
    let digest = reference
        .digest()
        .map(|d| d.to_string())
        .unwrap_or_else(|| "N/A".to_string());

    Ok(ImageDetails::new(
        reference_str.to_string(),
        digest,
        manifest_type,
        size,
        platforms,
        layers,
    ))
}

/// Get the registry URL from config or use default
pub(crate) fn get_registry_url() -> Result<String, String> {
    let config_path = config::get_config_path();

    // Try to load config
    let config = match config::Config::load(&config_path) {
        Ok(cfg) => cfg,
        Err(_) => {
            // No config, use default
            return Ok("http://localhost:5000".to_string());
        }
    };

    // Get default registry
    if let Some(default_name) = config.registries.default {
        // Find the registry by name
        if let Some(registry) = config
            .registries
            .list
            .iter()
            .find(|r| r.name == default_name)
        {
            return Ok(registry.url.clone());
        }
    }

    // No default set, use first registry if available
    if let Some(registry) = config.registries.list.first() {
        return Ok(registry.url.clone());
    }

    // Fall back to localhost:5000
    Ok("http://localhost:5000".to_string())
}

/// Get the cache directory for a specific registry
///
/// Creates a per-registry subdirectory under the main cache directory.
/// The subdirectory name is derived from the registry URL to ensure uniqueness.
///
/// # Arguments
///
/// * `registry_url` - The registry URL
///
/// # Returns
///
/// Returns the cache directory path for the registry
pub(crate) fn get_registry_cache_dir(registry_url: &str) -> Result<std::path::PathBuf, String> {
    let config_path = config::get_config_path();

    // Load config to get cache_dir
    let cache_base = if let Ok(cfg) = config::Config::load(&config_path) {
        std::path::PathBuf::from(cfg.cache_dir)
    } else {
        // Use default if config doesn't exist
        config::get_default_cache_dir()
    };

    // Create a safe directory name from the registry URL
    // Replace special characters with underscores
    let safe_name = registry_url
        .replace("://", "_")
        .replace(['/', ':', '.'], "_");

    Ok(cache_base.join(safe_name))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
