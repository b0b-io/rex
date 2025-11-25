//! Tag and repository metadata fetching logic.
//!
//! This module provides core functionality for fetching tag metadata (manifests,
//! configurations, sizes, platforms) and repository metadata (tag counts, sizes,
//! timestamps) in parallel. Used by both CLI and TUI.

use std::path::Path;
use std::str::FromStr;

use librex::{Credentials, Rex};
use rayon::prelude::*;

use super::types::{RepositoryItem, TagInfo};

/// Tag metadata fetcher.
///
/// Provides parallel tag metadata fetching with caching and authentication support.
/// This is the shared implementation used by both CLI and TUI.
///
/// # Examples
///
/// ```no_run
/// use rex::image::TagMetadataFetcher;
/// use std::path::Path;
///
/// let fetcher = TagMetadataFetcher::new(
///     "localhost:5000".to_string(),
///     Path::new("/tmp/cache"),
///     None,
///     8,
/// );
///
/// let tags = fetcher.fetch_tags("alpine").unwrap();
/// ```
pub struct TagMetadataFetcher {
    registry_url: String,
    cache_dir: std::path::PathBuf,
    credentials: Option<Credentials>,
    concurrency: usize,
}

impl TagMetadataFetcher {
    /// Create a new tag metadata fetcher.
    ///
    /// # Arguments
    ///
    /// * `registry_url` - The registry URL to connect to
    /// * `cache_dir` - Cache directory for storing fetched data
    /// * `credentials` - Optional credentials for authentication
    /// * `concurrency` - Maximum number of parallel connections
    pub fn new(
        registry_url: String,
        cache_dir: &Path,
        credentials: Option<Credentials>,
        concurrency: usize,
    ) -> Self {
        Self {
            registry_url,
            cache_dir: cache_dir.to_path_buf(),
            credentials,
            concurrency,
        }
    }

    /// Fetch tag metadata for all tags in a repository.
    ///
    /// This function:
    /// 1. Fetches the list of tags for the repository
    /// 2. Fetches manifest and config for each tag in parallel
    /// 3. Extracts size, platform, and created timestamp
    /// 4. Sorts results by created timestamp (newest first)
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository name to fetch tags from
    ///
    /// # Returns
    ///
    /// Returns a vector of TagInfo structs sorted by creation time (newest first).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot connect to registry
    /// - Cannot fetch tag list
    /// - Thread pool creation fails
    pub fn fetch_tags(&self, repository: &str) -> Result<Vec<TagInfo>, String> {
        // Build Rex client with cache and credentials
        let mut builder = Rex::builder()
            .registry_url(&self.registry_url)
            .with_cache(&self.cache_dir);

        if let Some(ref creds) = self.credentials {
            builder = builder.with_credentials(creds.clone());
        }

        let mut rex = builder
            .build()
            .map_err(|e| format!("Failed to connect to registry: {}", e))?;

        // Fetch tag names
        let tags = rex
            .list_tags(repository)
            .map_err(|e| format!("Failed to list tags: {}", e))?;

        if tags.is_empty() {
            return Ok(Vec::new());
        }

        // Configure thread pool with concurrency limit
        let concurrency = self.concurrency.min(tags.len()).max(1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(concurrency)
            .build()
            .map_err(|e| format!("Failed to create thread pool: {}", e))?;

        // Clone data for parallel access
        let registry_url = self.registry_url.clone();
        let cache_dir = self.cache_dir.clone();
        let credentials = self.credentials.clone();
        let repository = repository.to_string();

        // Fetch metadata for all tags in parallel
        let mut tag_infos: Vec<TagInfo> = pool.install(|| {
            tags.par_iter()
                .filter_map(|tag| {
                    fetch_single_tag_metadata(
                        &registry_url,
                        &repository,
                        tag,
                        &cache_dir,
                        credentials.clone(),
                    )
                })
                .collect()
        });

        // Sort by created timestamp in reverse chronological order (newest first)
        tag_infos.sort_by(|a, b| match (&a.created_timestamp, &b.created_timestamp) {
            (Some(time_a), Some(time_b)) => time_b.cmp(time_a), // Reverse order
            (Some(_), None) => std::cmp::Ordering::Less,        // With timestamp comes first
            (None, Some(_)) => std::cmp::Ordering::Greater,     // Without timestamp goes last
            (None, None) => a.tag.cmp(&b.tag),                  // Sort by name if both None
        });

        Ok(tag_infos)
    }
}

/// Fetch metadata for a single tag.
///
/// This is the per-thread worker function that fetches manifest and config
/// for a single tag. It mirrors the CLI implementation exactly.
///
/// # Arguments
///
/// * `registry_url` - The registry URL
/// * `repository` - The repository name
/// * `tag` - The tag name
/// * `cache_dir` - Cache directory path
/// * `credentials` - Optional credentials
///
/// # Returns
///
/// Returns Some(TagInfo) on success, None on error (logged to stderr).
fn fetch_single_tag_metadata(
    registry_url: &str,
    repository: &str,
    tag: &str,
    cache_dir: &Path,
    credentials: Option<Credentials>,
) -> Option<TagInfo> {
    // Create per-thread Rex instance with cache and credentials
    let mut builder = Rex::builder()
        .registry_url(registry_url)
        .with_cache(cache_dir);

    if let Some(ref creds) = credentials {
        builder = builder.with_credentials(creds.clone());
    }

    let mut thread_rex = match builder.build() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Warning: Failed to connect for tag {}: {}", tag, e);
            return Some(TagInfo::new(
                tag.to_string(),
                "N/A".to_string(),
                0,
                None,
                vec![],
            ));
        }
    };

    // Fetch manifest for this tag
    let reference = format!("{}:{}", repository, tag);

    let (manifest_or_index, digest) = match thread_rex.get_manifest(&reference) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Warning: Failed to fetch manifest for {}: {}", reference, e);
            return Some(TagInfo::new(
                tag.to_string(),
                "N/A".to_string(),
                0,
                None,
                vec![],
            ));
        }
    };

    // Extract details based on manifest type
    let (size, platforms, created) = match &manifest_or_index {
        librex::oci::ManifestOrIndex::Manifest(manifest) => {
            // Single-platform image - sum layer sizes
            let total_size: u64 = manifest.layers().iter().map(|layer| layer.size()).sum();

            // Get config to extract platform and created timestamp
            let config_digest_str = manifest.config().digest().to_string();
            let config_digest = match librex::digest::Digest::from_str(&config_digest_str) {
                Ok(d) => d,
                Err(_) => {
                    return Some(TagInfo::new(
                        tag.to_string(),
                        digest,
                        total_size,
                        None,
                        vec![],
                    ));
                }
            };

            let config_bytes = match thread_rex.get_blob(repository, &config_digest) {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Some(TagInfo::new(
                        tag.to_string(),
                        digest,
                        total_size,
                        None,
                        vec![],
                    ));
                }
            };

            let config: librex::oci::ImageConfiguration =
                match serde_json::from_slice(&config_bytes) {
                    Ok(c) => c,
                    Err(_) => {
                        return Some(TagInfo::new(
                            tag.to_string(),
                            digest,
                            total_size,
                            None,
                            vec![],
                        ));
                    }
                };

            // Extract platform (os/architecture)
            let platform = vec![format!("{}/{}", config.os(), config.architecture())];

            // Parse created timestamp from ISO 8601 string to DateTime
            let created_ts = config.created().as_ref().and_then(|ts| {
                chrono::DateTime::parse_from_rfc3339(ts)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            });

            (total_size, platform, created_ts)
        }
        librex::oci::ManifestOrIndex::Index(index) => {
            // Multi-platform image - sum manifest sizes from index
            let total_size: u64 = index.manifests().iter().map(|m| m.size()).sum();

            // Extract platforms from manifest descriptors
            let platforms: Vec<String> = index
                .manifests()
                .iter()
                .filter_map(|m| {
                    m.platform()
                        .as_ref()
                        .map(|p| format!("{}/{}", p.os(), p.architecture()))
                })
                .collect();

            // Multi-platform indexes don't have a single created timestamp
            (total_size, platforms, None)
        }
    };

    // Create TagInfo with formatted display values
    Some(TagInfo::new(
        tag.to_string(),
        digest,
        size,
        created,
        platforms,
    ))
}

/// Repository metadata fetcher.
///
/// Provides parallel repository metadata fetching with caching and authentication support.
/// This is the shared implementation used by both CLI and TUI.
///
/// # Examples
///
/// ```no_run
/// use rex::image::RepositoryMetadataFetcher;
/// use std::path::Path;
///
/// let fetcher = RepositoryMetadataFetcher::new(
///     "localhost:5000".to_string(),
///     Path::new("/tmp/cache"),
///     None,
///     8,
/// );
///
/// let repos = fetcher.fetch_repositories().unwrap();
/// ```
pub struct RepositoryMetadataFetcher {
    registry_url: String,
    cache_dir: std::path::PathBuf,
    credentials: Option<Credentials>,
    concurrency: usize,
}

impl RepositoryMetadataFetcher {
    /// Create a new repository metadata fetcher.
    ///
    /// # Arguments
    ///
    /// * `registry_url` - The registry URL to connect to
    /// * `cache_dir` - Cache directory for storing fetched data
    /// * `credentials` - Optional credentials for authentication
    /// * `concurrency` - Maximum number of parallel connections
    pub fn new(
        registry_url: String,
        cache_dir: &Path,
        credentials: Option<Credentials>,
        concurrency: usize,
    ) -> Self {
        Self {
            registry_url,
            cache_dir: cache_dir.to_path_buf(),
            credentials,
            concurrency,
        }
    }

    /// Fetch repository metadata for all repositories in the registry.
    ///
    /// This function:
    /// 1. Fetches the list of repositories from the registry
    /// 2. For each repository in parallel:
    ///    - Fetches tag list to get count
    ///    - Fetches manifest for the last tag to get size and timestamp
    /// 3. Returns RepositoryItem structs with formatted display values
    ///
    /// # Arguments
    ///
    /// * `progress_callback` - Optional callback invoked after each repository is processed
    ///
    /// # Returns
    ///
    /// Returns a vector of RepositoryItem structs.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot connect to registry
    /// - Cannot fetch repository list
    /// - Thread pool creation fails
    pub fn fetch_repositories<F>(
        &self,
        progress_callback: Option<F>,
    ) -> Result<Vec<RepositoryItem>, String>
    where
        F: Fn() + Send + Sync,
    {
        // Build Rex client with cache and credentials
        let mut builder = Rex::builder()
            .registry_url(&self.registry_url)
            .with_cache(&self.cache_dir);

        if let Some(ref creds) = self.credentials {
            builder = builder.with_credentials(creds.clone());
        }

        let mut rex = builder
            .build()
            .map_err(|e| format!("Failed to connect to registry: {}", e))?;

        // Fetch repository list
        let repos = rex
            .list_repositories()
            .map_err(|e| format!("Failed to list repositories: {}", e))?;

        if repos.is_empty() {
            return Ok(Vec::new());
        }

        // Configure thread pool for parallel fetching
        let concurrency = self.concurrency.min(repos.len()).max(1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(concurrency)
            .build()
            .map_err(|e| format!("Failed to create thread pool: {}", e))?;

        // Clone data for parallel access
        let registry_url = self.registry_url.clone();
        let cache_dir = self.cache_dir.clone();
        let credentials = self.credentials.clone();

        // Fetch metadata for all repositories in parallel
        let results: Vec<RepositoryItem> = pool.install(|| {
            repos
                .par_iter()
                .map(|repo| {
                    let result = fetch_single_repository_metadata(
                        &registry_url,
                        repo,
                        &cache_dir,
                        credentials.clone(),
                    );

                    // Invoke progress callback if provided
                    if let Some(ref callback) = progress_callback {
                        callback();
                    }

                    result
                })
                .collect()
        });

        Ok(results)
    }
}

/// Fetch metadata for a single repository.
///
/// This is the per-thread worker function that fetches tag list, manifest,
/// and config for a single repository to extract tag count, size, and last updated.
///
/// # Arguments
///
/// * `registry_url` - The registry URL
/// * `repository` - The repository name
/// * `cache_dir` - Cache directory path
/// * `credentials` - Optional credentials
///
/// # Returns
///
/// Returns RepositoryItem with metadata or default values on error.
fn fetch_single_repository_metadata(
    registry_url: &str,
    repository: &str,
    cache_dir: &Path,
    credentials: Option<Credentials>,
) -> RepositoryItem {
    // Create per-thread Rex instance with cache and credentials
    let mut builder = Rex::builder()
        .registry_url(registry_url)
        .with_cache(cache_dir);

    if let Some(ref creds) = credentials {
        builder = builder.with_credentials(creds.clone());
    }

    let mut thread_rex = match builder.build() {
        Ok(r) => r,
        Err(_) => {
            // On connection error, return repo with no data
            return RepositoryItem::new(repository.to_string(), 0, 0, None);
        }
    };

    // Fetch tags to get count (uses cache if available)
    let tags = match thread_rex.list_tags(repository) {
        Ok(t) => t,
        Err(_) => {
            // On error, return repo with no data
            return RepositoryItem::new(repository.to_string(), 0, 0, None);
        }
    };

    let tag_count = tags.len();

    // Get size and date from most recent tag (last in list)
    let (total_size, last_updated) = if let Some(last_tag) = tags.last() {
        match thread_rex.get_manifest(&format!("{}:{}", repository, last_tag)) {
            Ok((manifest, _digest)) => {
                use librex::oci::ManifestOrIndex;

                let size = match &manifest {
                    ManifestOrIndex::Manifest(m) => {
                        // Sum layer sizes
                        m.layers().iter().map(|l| l.size()).sum()
                    }
                    ManifestOrIndex::Index(idx) => {
                        // Sum manifest sizes in index
                        idx.manifests().iter().map(|m| m.size()).sum()
                    }
                };

                // Try to get created date from config (manifest only)
                let created = if let ManifestOrIndex::Manifest(m) = &manifest {
                    // Get config blob and parse created timestamp
                    m.config()
                        .digest()
                        .to_string()
                        .parse::<librex::digest::Digest>()
                        .ok()
                        .and_then(|digest| thread_rex.get_blob(repository, &digest).ok())
                        .and_then(|bytes| {
                            serde_json::from_slice::<librex::oci::ImageConfiguration>(&bytes).ok()
                        })
                        .and_then(|config| {
                            config.created().as_ref().and_then(|ts| {
                                chrono::DateTime::parse_from_rfc3339(ts)
                                    .ok()
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                            })
                        })
                } else {
                    None
                };

                (size, created)
            }
            Err(_) => (0, None), // On manifest fetch error, use defaults
        }
    } else {
        (0, None) // No tags
    };

    RepositoryItem::new(repository.to_string(), tag_count, total_size, last_updated)
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod tests;
