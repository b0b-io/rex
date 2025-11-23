//! Background worker functions for Rex TUI.
//!
//! Provides worker functions that perform blocking I/O operations in background
//! threads, sending results back to the UI thread via channels.

use std::path::Path;
use std::sync::mpsc::Sender;

use librex::{Credentials, Rex};
use rayon::prelude::*;

use super::Result;
use super::app::Message;
use super::views::repos::RepositoryItem;

/// Fetch repositories with metadata (tag counts).
///
/// This function fetches the repository list and computes tag counts
/// by fetching tags in parallel. Follows the pattern used in the CLI's
/// `list_images` function.
///
/// Each thread gets its own Rex instance with cache and credentials, allowing
/// parallel fetching while utilizing the cache for better performance.
///
/// # Arguments
///
/// * `registry_url` - The URL of the registry to query
/// * `cache_dir` - The cache directory path
/// * `credentials` - Optional credentials for authentication
/// * `tx` - The channel sender for sending results back to the UI thread
/// * `concurrency` - Maximum number of parallel connections
///
/// # Examples
///
/// ```no_run
/// use std::sync::mpsc::channel;
/// use std::path::Path;
/// use rex::tui::worker::fetch_repositories;
///
/// let (tx, rx) = channel();
/// let cache_dir = Path::new("/tmp/cache");
/// std::thread::spawn(move || {
///     fetch_repositories("localhost:5000".to_string(), cache_dir, None, tx, 8);
/// });
/// ```
pub fn fetch_repositories(
    registry_url: String,
    cache_dir: &Path,
    credentials: Option<Credentials>,
    tx: Sender<Message>,
    concurrency: usize,
) {
    let cache_dir_owned = cache_dir.to_path_buf();

    let result = (|| -> Result<Vec<RepositoryItem>> {
        // Build Rex client with cache and credentials
        let mut builder = Rex::builder()
            .registry_url(&registry_url)
            .with_cache(&cache_dir_owned);

        if let Some(ref creds) = credentials {
            builder = builder.with_credentials(creds.clone());
        }

        let mut rex = builder.build()?;

        // Fetch repository list
        let repos = rex.list_repositories()?;

        if repos.is_empty() {
            return Ok(Vec::new());
        }

        // Configure thread pool for parallel tag fetching
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(concurrency.min(repos.len()).max(1))
            .build()
            .map_err(|e| {
                Box::new(librex::RexError::validation(format!(
                    "Failed to create thread pool: {}",
                    e
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

        // Clone data for parallel access
        let registry_url_clone = registry_url.clone();
        let cache_dir_clone = cache_dir_owned.clone();
        let credentials_clone = credentials.clone();

        // Fetch tag counts for all repositories in parallel
        let results: Vec<RepositoryItem> = pool.install(|| {
            repos
                .par_iter()
                .map(|repo| {
                    // Each thread gets its own Rex instance with cache
                    let mut builder = Rex::builder()
                        .registry_url(&registry_url_clone)
                        .with_cache(cache_dir_clone.clone());

                    if let Some(ref creds) = credentials_clone {
                        builder = builder.with_credentials(creds.clone());
                    }

                    let mut thread_rex = match builder.build() {
                        Ok(r) => r,
                        Err(_) => {
                            // On connection error, return repo with 0 tags
                            return RepositoryItem {
                                name: repo.clone(),
                                tag_count: 0,
                                total_size: 0,
                                last_updated: None,
                            };
                        }
                    };

                    // Fetch tags to get count (uses cache if available)
                    let tags = match thread_rex.list_tags(repo) {
                        Ok(t) => t,
                        Err(_) => {
                            // On error, return repo with no data
                            return RepositoryItem {
                                name: repo.clone(),
                                tag_count: 0,
                                total_size: 0,
                                last_updated: None,
                            };
                        }
                    };

                    let tag_count = tags.len();

                    // Get size and date from most recent tag (last in list)
                    let (total_size, last_updated) = if let Some(last_tag) = tags.last() {
                        match thread_rex.get_manifest(&format!("{}:{}", repo, last_tag)) {
                            Ok((manifest, _digest)) => {
                                use librex::ManifestOrIndex;

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
                                        .parse::<librex::Digest>()
                                        .ok()
                                        .and_then(|digest| thread_rex.get_blob(repo, &digest).ok())
                                        .and_then(|bytes| {
                                            serde_json::from_slice::<
                                                    librex::oci::ImageConfiguration,
                                                >(
                                                    &bytes
                                                )
                                                .ok()
                                        })
                                        .and_then(|config| {
                                            config.created().as_ref().map(|ts| ts.to_string())
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

                    RepositoryItem {
                        name: repo.clone(),
                        tag_count,
                        total_size,
                        last_updated,
                    }
                })
                .collect()
        });

        Ok(results)
    })();

    // Send result back to UI thread
    let _ = tx.send(Message::RepositoriesLoaded(result));
}

/// Fetch the list of tags for a specific repository.
///
/// This function is meant to be called in a background thread via `App::spawn_worker`.
///
/// # Arguments
///
/// * `registry_url` - The URL of the registry to query
/// * `repository` - The name of the repository
/// * `cache_dir` - The cache directory path
/// * `credentials` - Optional credentials for authentication
/// * `tx` - The channel sender for sending the result back to the UI thread
///
/// # Examples
///
/// ```no_run
/// use std::sync::mpsc::channel;
/// use std::path::Path;
/// use rex::tui::worker::fetch_tags;
///
/// let (tx, rx) = channel();
/// let repo = "alpine".to_string();
/// let cache_dir = Path::new("/tmp/cache");
/// std::thread::spawn(move || {
///     fetch_tags("localhost:5000".to_string(), repo, cache_dir, None, tx);
/// });
/// ```
#[allow(dead_code)] // TODO: Remove when integrated with views
pub fn fetch_tags(
    registry_url: String,
    repository: String,
    cache_dir: &Path,
    credentials: Option<Credentials>,
    tx: Sender<Message>,
) {
    let repo_clone = repository.clone();

    let result = (|| -> Result<Vec<String>> {
        // Build Rex client with cache and credentials
        let mut builder = Rex::builder()
            .registry_url(&registry_url)
            .with_cache(cache_dir);

        if let Some(creds) = credentials {
            builder = builder.with_credentials(creds);
        }

        let mut rex = builder.build()?;

        // Fetch tags for the repository
        let tags = rex.list_tags(&repository)?;

        Ok(tags)
    })();

    // Send result back to UI thread
    let _ = tx.send(Message::TagsLoaded(repo_clone, result));
}

/// Fetch the manifest and configuration for a specific image.
///
/// This function is meant to be called in a background thread via `App::spawn_worker`.
/// It fetches both the manifest and the configuration blob, sending them as separate messages.
///
/// # Arguments
///
/// * `registry_url` - The URL of the registry to query
/// * `repository` - The name of the repository
/// * `tag` - The tag of the image
/// * `cache_dir` - The cache directory path
/// * `credentials` - Optional credentials for authentication
/// * `tx` - The channel sender for sending the result back to the UI thread
///
/// # Examples
///
/// ```no_run
/// use std::sync::mpsc::channel;
/// use std::path::Path;
/// use rex::tui::worker::fetch_manifest_and_config;
///
/// let (tx, rx) = channel();
/// let repo = "alpine".to_string();
/// let tag = "latest".to_string();
/// let cache_dir = Path::new("/tmp/cache");
/// std::thread::spawn(move || {
///     fetch_manifest_and_config("localhost:5000".to_string(), repo, tag, cache_dir, None, tx);
/// });
/// ```
pub fn fetch_manifest_and_config(
    registry_url: String,
    repository: String,
    tag: String,
    cache_dir: &Path,
    credentials: Option<Credentials>,
    tx: Sender<Message>,
) {
    let repo_clone = repository.clone();
    let tag_clone = tag.clone();

    // Build Rex client with cache and credentials
    let mut builder = Rex::builder()
        .registry_url(&registry_url)
        .with_cache(cache_dir);

    if let Some(creds) = credentials {
        builder = builder.with_credentials(creds);
    }

    let mut rex = match builder.build() {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(Message::ManifestLoaded(
                repo_clone.clone(),
                tag_clone.clone(),
                Box::new(Err(Box::new(e))),
            ));
            return;
        }
    };

    // Build reference string (repository:tag)
    let reference_str = format!("{}:{}", repository, tag);

    // Fetch manifest
    let manifest_result = rex.get_manifest(&reference_str);

    match manifest_result {
        Ok((manifest_or_index, _digest)) => {
            // Send manifest
            let _ = tx.send(Message::ManifestLoaded(
                repo_clone.clone(),
                tag_clone.clone(),
                Box::new(Ok(manifest_or_index.clone())),
            ));

            // Try to fetch config blob for single-platform manifests
            if let librex::ManifestOrIndex::Manifest(manifest) = &manifest_or_index {
                let config_digest_str = manifest.config().digest().to_string();

                // Parse digest string
                let config_digest = match config_digest_str.parse::<librex::Digest>() {
                    Ok(d) => d,
                    Err(e) => {
                        let _ = tx.send(Message::ConfigLoaded(
                            repo_clone,
                            tag_clone,
                            Box::new(Err(Box::new(e))),
                        ));
                        return;
                    }
                };

                // Fetch config blob
                match rex.get_blob(&repository, &config_digest) {
                    Ok(config_bytes) => {
                        // Parse config JSON
                        match serde_json::from_slice::<librex::oci::ImageConfiguration>(
                            &config_bytes,
                        ) {
                            Ok(config) => {
                                let _ = tx.send(Message::ConfigLoaded(
                                    repo_clone,
                                    tag_clone,
                                    Box::new(Ok(config)),
                                ));
                            }
                            Err(e) => {
                                let _ = tx.send(Message::ConfigLoaded(
                                    repo_clone,
                                    tag_clone,
                                    Box::new(Err(Box::new(
                                        librex::RexError::validation_with_source(
                                            "Failed to parse config JSON",
                                            e,
                                        ),
                                    ))),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Message::ConfigLoaded(
                            repo_clone,
                            tag_clone,
                            Box::new(Err(Box::new(e))),
                        ));
                    }
                }
            } else {
                // For manifest indexes, there's no single config
                // Clear loading state by sending an error (which is expected)
                let _ = tx.send(Message::ConfigLoaded(
                    repo_clone,
                    tag_clone,
                    Box::new(Err(Box::new(librex::RexError::validation(
                        "No config for manifest index",
                    )))),
                ));
            }
        }
        Err(e) => {
            let _ = tx.send(Message::ManifestLoaded(
                repo_clone,
                tag_clone,
                Box::new(Err(Box::new(e))),
            ));
        }
    }
}

#[cfg(test)]
#[path = "worker_tests.rs"]
mod tests;
