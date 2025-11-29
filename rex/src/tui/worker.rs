//! Background worker functions for Rex TUI.
//!
//! Provides worker functions that perform blocking I/O operations in background
//! threads, sending results back to the UI thread via channels.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;

use librex::{Credentials, Rex};

use super::Result;
use super::app::Message;

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

    // First, get the repository count for progress tracking
    // Build a Rex client to fetch the repository list
    let mut builder = Rex::builder()
        .registry_url(&registry_url)
        .with_cache(&cache_dir_owned);

    if let Some(ref creds) = credentials {
        builder = builder.with_credentials(creds.clone());
    }

    let mut rex = match builder.build() {
        Ok(rex) => rex,
        Err(e) => {
            let result: Result<Vec<crate::image::RepositoryItem>> = Err(Box::new(
                std::io::Error::other(format!("Failed to connect to registry: {}", e)),
            ));
            let _ = tx.send(Message::RepositoriesLoaded(result));
            return;
        }
    };

    // Fetch repository list to get the total count
    let repo_list = match rex.list_repositories() {
        Ok(repos) => repos,
        Err(e) => {
            let result: Result<Vec<crate::image::RepositoryItem>> = Err(Box::new(
                std::io::Error::other(format!("Failed to list repositories: {}", e)),
            ));
            let _ = tx.send(Message::RepositoriesLoaded(result));
            return;
        }
    };

    let total = repo_list.len();

    // Send initial progress (0/total)
    let _ = tx.send(Message::RepositoryProgress(0, total));

    // If no repositories, send empty result
    if total == 0 {
        let _ = tx.send(Message::RepositoriesLoaded(Ok(Vec::new())));
        return;
    }

    // Create fetcher for parallel metadata fetching
    let fetcher = crate::image::RepositoryMetadataFetcher::new(
        registry_url,
        &cache_dir_owned,
        credentials,
        concurrency,
    );

    // Track progress with atomic counter
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let tx_clone = tx.clone();

    // Fetch with progress callback
    let result = fetcher
        .fetch_repositories(Some(move || {
            let current = counter_clone.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = tx_clone.send(Message::RepositoryProgress(current, total));
        }))
        .map_err(|e| {
            Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error + Send + Sync>
        });

    // Send final result back to UI thread
    let _ = tx.send(Message::RepositoriesLoaded(result));
}

/// Fetch tags with metadata for a repository.
///
/// This function uses the shared TagMetadataFetcher to fetch full tag metadata
/// (digest, size, platforms, created timestamp) in parallel.
///
/// # Arguments
///
/// * `registry_url` - The URL of the registry to query
/// * `repository` - The name of the repository
/// * `cache_dir` - The cache directory path
/// * `credentials` - Optional credentials for authentication
/// * `tx` - The channel sender for sending the result back to the UI thread
/// * `concurrency` - Maximum number of parallel connections
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
///     fetch_tags("localhost:5000".to_string(), repo, cache_dir, None, tx, 8);
/// });
/// ```
pub fn fetch_tags(
    registry_url: String,
    repository: String,
    cache_dir: &Path,
    credentials: Option<Credentials>,
    tx: Sender<Message>,
    concurrency: usize,
) {
    let repo_clone = repository.clone();

    let result = (|| -> Result<Vec<crate::image::TagInfo>> {
        // Use shared TagMetadataFetcher for consistent behavior with CLI
        let fetcher = crate::image::TagMetadataFetcher::new(
            registry_url,
            cache_dir,
            credentials,
            concurrency,
        );

        // Fetch tags with full metadata
        let tags = fetcher.fetch_tags(&repository).map_err(|e| {
            Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error + Send + Sync>
        })?;

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
