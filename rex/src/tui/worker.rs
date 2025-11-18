//! Background worker functions for Rex TUI.
//!
//! Provides worker functions that perform blocking I/O operations in background
//! threads, sending results back to the UI thread via channels.

use std::path::Path;
use std::sync::mpsc::Sender;

use librex::{Credentials, Rex};

use super::Result;
use super::app::Message;

/// Fetch the list of repositories from a registry.
///
/// This function is meant to be called in a background thread via `App::spawn_worker`.
///
/// # Arguments
///
/// * `registry_url` - The URL of the registry to query
/// * `cache_dir` - The cache directory path
/// * `credentials` - Optional credentials for authentication
/// * `tx` - The channel sender for sending the result back to the UI thread
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
///     fetch_repositories("localhost:5000".to_string(), cache_dir, None, tx);
/// });
/// ```
pub fn fetch_repositories(
    registry_url: String,
    cache_dir: &Path,
    credentials: Option<Credentials>,
    tx: Sender<Message>,
) {
    let result = (|| -> Result<Vec<String>> {
        // Build Rex client with cache and credentials
        let mut builder = Rex::builder()
            .registry_url(&registry_url)
            .with_cache(cache_dir);

        if let Some(creds) = credentials {
            builder = builder.with_credentials(creds);
        }

        let mut rex = builder.build()?;

        // Fetch repository list
        let repos = rex.list_repositories()?;

        Ok(repos)
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

/// Fetch the manifest for a specific image.
///
/// This function is meant to be called in a background thread via `App::spawn_worker`.
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
/// use rex::tui::worker::fetch_manifest;
///
/// let (tx, rx) = channel();
/// let repo = "alpine".to_string();
/// let tag = "latest".to_string();
/// let cache_dir = Path::new("/tmp/cache");
/// std::thread::spawn(move || {
///     fetch_manifest("localhost:5000".to_string(), repo, tag, cache_dir, None, tx);
/// });
/// ```
#[allow(dead_code)] // TODO: Remove when integrated with views
pub fn fetch_manifest(
    registry_url: String,
    repository: String,
    tag: String,
    cache_dir: &Path,
    credentials: Option<Credentials>,
    tx: Sender<Message>,
) {
    let repo_clone = repository.clone();
    let tag_clone = tag.clone();

    let result = (|| -> Result<Vec<u8>> {
        // Build Rex client with cache and credentials
        let mut builder = Rex::builder()
            .registry_url(&registry_url)
            .with_cache(cache_dir);

        if let Some(creds) = credentials {
            builder = builder.with_credentials(creds);
        }

        let mut rex = builder.build()?;

        // Build reference string (repository:tag)
        let reference_str = format!("{}:{}", repository, tag);

        // Fetch manifest
        let _manifest = rex.get_manifest(&reference_str)?;

        // TODO: Properly serialize manifest when views are implemented
        // For now, return empty bytes as placeholder
        Ok(vec![])
    })();

    // Send result back to UI thread
    let _ = tx.send(Message::ManifestLoaded(repo_clone, tag_clone, result));
}

#[cfg(test)]
#[path = "worker_tests.rs"]
mod tests;
