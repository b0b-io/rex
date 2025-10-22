use crate::config;
use crate::format::Formattable;
use librex::auth::CredentialStore;
use serde::Serialize;
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
