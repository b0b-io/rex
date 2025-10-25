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

/// Layer information for detailed inspection
#[derive(Debug, Serialize)]
pub struct LayerInfo {
    pub digest: String,
    pub size: u64,
    pub media_type: String,
}

/// History entry for image inspection
#[derive(Debug, Serialize)]
pub struct HistoryEntry {
    pub created: Option<String>,
    pub created_by: Option<String>,
    pub empty_layer: bool,
}

/// Complete inspection data for an image
#[derive(Debug, Serialize)]
pub struct ImageInspect {
    /// Full reference (name:tag or name@digest)
    pub reference: String,
    /// Registry URL
    pub registry: String,
    /// Manifest digest
    pub manifest_digest: String,
    /// Manifest type
    pub manifest_type: String,
    /// Config digest
    pub config_digest: String,
    /// Total size in bytes
    pub size: u64,
    /// Architecture
    pub architecture: String,
    /// Operating system
    pub os: String,
    /// Created timestamp
    pub created: Option<String>,
    /// Environment variables
    pub env: Vec<String>,
    /// Entrypoint
    pub entrypoint: Option<Vec<String>>,
    /// Command
    pub cmd: Option<Vec<String>>,
    /// Working directory
    pub working_dir: Option<String>,
    /// User
    pub user: Option<String>,
    /// Labels
    pub labels: std::collections::HashMap<String, String>,
    /// Exposed ports
    pub exposed_ports: Vec<String>,
    /// Volumes
    pub volumes: Vec<String>,
    /// Layers with details
    pub layers: Vec<LayerInfo>,
    /// History entries
    pub history: Vec<HistoryEntry>,
    /// RootFS diff IDs
    pub rootfs_diff_ids: Vec<String>,
}

impl Formattable for ImageInspect {
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

        // Basic info
        output.push_str(&format!("Image: {}\n", self.reference));
        output.push_str(&format!("Digest: {}\n", self.manifest_digest));
        output.push_str(&format!("Registry: {}\n", self.registry));
        output.push_str(&format!("Type: {}\n", self.manifest_type));
        output.push_str(&format!("Total Size: {}\n", format_bytes(self.size)));
        output.push('\n');

        output.push_str(&format!("Manifest Digest: {}\n", self.manifest_digest));
        output.push_str(&format!("Config Digest: {}\n", self.config_digest));
        output.push('\n');

        // Configuration
        output.push_str("Configuration:\n");
        output.push_str(&format!("  Architecture: {}\n", self.architecture));
        output.push_str(&format!("  OS: {}\n", self.os));
        if let Some(created) = &self.created {
            output.push_str(&format!("  Created: {}\n", created));
        }
        output.push('\n');

        // Config details
        output.push_str("  Config:\n");
        if let Some(user) = &self.user {
            output.push_str(&format!("    User: {}\n", user));
        } else {
            output.push_str("    User: (empty)\n");
        }

        if !self.env.is_empty() {
            output.push_str("    Env:\n");
            for env in &self.env {
                output.push_str(&format!("      - {}\n", env));
            }
        }

        if let Some(entrypoint) = &self.entrypoint {
            output.push_str("    Entrypoint:\n");
            for entry in entrypoint {
                output.push_str(&format!("      - {}\n", entry));
            }
        }

        if let Some(cmd) = &self.cmd {
            output.push_str("    Cmd:\n");
            for c in cmd {
                output.push_str(&format!("      - {}\n", c));
            }
        }

        if let Some(wd) = &self.working_dir {
            output.push_str(&format!("    WorkingDir: {}\n", wd));
        }

        if !self.exposed_ports.is_empty() {
            output.push_str("    ExposedPorts:\n");
            for port in &self.exposed_ports {
                output.push_str(&format!("      - {}\n", port));
            }
        }

        if !self.volumes.is_empty() {
            output.push_str("    Volumes:\n");
            for vol in &self.volumes {
                output.push_str(&format!("      - {}\n", vol));
            }
        }

        if !self.labels.is_empty() {
            output.push_str("\n  Labels:\n");
            for (key, value) in &self.labels {
                output.push_str(&format!("    {}: {}\n", key, value));
            }
        }

        // Layers
        output.push_str(&format!("\nLayers ({}):\n", self.layers.len()));
        for (i, layer) in self.layers.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, layer.digest));
            output.push_str(&format!(
                "     Size: {} ({})\n",
                format_bytes(layer.size),
                layer.size
            ));
            output.push_str(&format!("     Media Type: {}\n", layer.media_type));
        }

        // History
        if !self.history.is_empty() {
            output.push_str(&format!("\nHistory ({} entries):\n", self.history.len()));
            for (i, entry) in self.history.iter().enumerate() {
                output.push_str(&format!("  {}. ", i + 1));
                if let Some(created) = &entry.created {
                    output.push_str(&format!("Created: {}", created));
                }
                if entry.empty_layer {
                    output.push_str(" (empty layer)");
                }
                output.push('\n');
                if let Some(created_by) = &entry.created_by {
                    output.push_str(&format!("     {}\n", created_by));
                }
            }
        }

        // RootFS
        output.push_str("\nRootFS:\n");
        output.push_str("  Type: layers\n");
        output.push_str("  DiffIDs:\n");
        for diff_id in &self.rootfs_diff_ids {
            output.push_str(&format!("    - {}\n", diff_id));
        }

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

/// Get complete inspection details for a specific image reference
///
/// # Arguments
///
/// * `registry_url` - URL of the registry to query
/// * `reference_str` - Full image reference (e.g., "alpine:latest" or "alpine@sha256:...")
///
/// # Returns
///
/// Returns ImageInspect with complete manifest, config, layers, and history information
pub(crate) async fn get_image_inspect(
    registry_url: &str,
    reference_str: &str,
) -> Result<ImageInspect, String> {
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

    // Get the manifest
    let manifest_or_index = rex
        .get_manifest(reference_str)
        .await
        .map_err(|e| format!("Failed to fetch manifest: {}", e))?;

    // For now, we only support single-platform manifests for inspect
    // TODO: Add support for --platform flag to select from multi-platform images
    let manifest = match manifest_or_index {
        librex::oci::ManifestOrIndex::Manifest(m) => m,
        librex::oci::ManifestOrIndex::Index(_) => {
            return Err(
                "Multi-platform images not yet supported for inspect. Use 'show' command or specify --platform flag."
                    .to_string(),
            );
        }
    };

    // Get config blob
    let config_digest_str = manifest.config().digest().to_string();
    let config_digest = librex::digest::Digest::from_str(&config_digest_str)
        .map_err(|e| format!("Invalid config digest: {}", e))?;

    let config_bytes = rex
        .get_blob(reference.repository(), &config_digest)
        .await
        .map_err(|e| format!("Failed to fetch config blob: {}", e))?;

    let config: librex::oci::ImageConfiguration = serde_json::from_slice(&config_bytes)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Extract layer information
    let layers: Vec<LayerInfo> = manifest
        .layers()
        .iter()
        .map(|layer| LayerInfo {
            digest: layer.digest().to_string(),
            size: layer.size(),
            media_type: layer.media_type().to_string(),
        })
        .collect();

    // Calculate total size
    let total_size: u64 = layers.iter().map(|l| l.size).sum();

    // Extract history
    let history: Vec<HistoryEntry> = config
        .history()
        .as_ref()
        .map(|h| {
            h.iter()
                .map(|entry| HistoryEntry {
                    created: entry.created().as_ref().map(|c| c.to_string()),
                    created_by: entry.created_by().as_ref().map(|s| s.to_string()),
                    empty_layer: entry.empty_layer().unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default();

    // Extract environment variables
    let env = config
        .config()
        .as_ref()
        .and_then(|c| c.env().as_ref())
        .map(|e| e.to_vec())
        .unwrap_or_default();

    // Extract entrypoint
    let entrypoint = config
        .config()
        .as_ref()
        .and_then(|c| c.entrypoint().as_ref())
        .map(|e| e.to_vec());

    // Extract cmd
    let cmd = config
        .config()
        .as_ref()
        .and_then(|c| c.cmd().as_ref())
        .map(|c| c.to_vec());

    // Extract working directory
    let working_dir = config
        .config()
        .as_ref()
        .and_then(|c| c.working_dir().as_ref())
        .map(|s| s.to_string());

    // Extract user
    let user = config
        .config()
        .as_ref()
        .and_then(|c| c.user().as_ref())
        .map(|s| s.to_string());

    // Extract labels
    let labels = config
        .config()
        .as_ref()
        .and_then(|c| c.labels().as_ref())
        .cloned()
        .unwrap_or_default();

    // Extract exposed ports
    let exposed_ports = config
        .config()
        .as_ref()
        .and_then(|c| c.exposed_ports().as_ref())
        .map(|ports| ports.to_vec())
        .unwrap_or_default();

    // Extract volumes
    let volumes = config
        .config()
        .as_ref()
        .and_then(|c| c.volumes().as_ref())
        .map(|vols| vols.to_vec())
        .unwrap_or_default();

    // Extract RootFS diff IDs
    let rootfs_diff_ids = config
        .rootfs()
        .diff_ids()
        .iter()
        .map(|d| d.to_string())
        .collect();

    // Get manifest digest
    let manifest_digest = reference
        .digest()
        .map(|d| d.to_string())
        .unwrap_or_else(|| "N/A".to_string());

    Ok(ImageInspect {
        reference: reference_str.to_string(),
        registry: registry_url.to_string(),
        manifest_digest,
        manifest_type: "OCI Image Manifest".to_string(),
        config_digest: config_digest_str,
        size: total_size,
        architecture: config.architecture().to_string(),
        os: config.os().to_string(),
        created: config.created().as_ref().map(|c| c.to_string()),
        env,
        entrypoint,
        cmd,
        working_dir,
        user,
        labels,
        exposed_ports,
        volumes,
        layers,
        history,
        rootfs_diff_ids,
    })
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
    config::get_registry_cache_dir(registry_url)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
