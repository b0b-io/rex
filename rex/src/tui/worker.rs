//! Background worker functions for Rex TUI.
//!
//! Provides worker functions that perform blocking I/O operations in background
//! threads, sending results back to the UI thread via channels.

use std::sync::mpsc::Sender;

use librex::Rex;

use super::Result;
use super::app::Message;

/// Fetch the list of repositories from a registry.
///
/// This function is meant to be called in a background thread via `App::spawn_worker`.
///
/// # Arguments
///
/// * `registry_url` - The URL of the registry to query
/// * `tx` - The channel sender for sending the result back to the UI thread
///
/// # Examples
///
/// ```no_run
/// use std::sync::mpsc::channel;
/// use rex::tui::worker::fetch_repositories;
///
/// let (tx, rx) = channel();
/// std::thread::spawn(move || {
///     fetch_repositories("localhost:5000".to_string(), tx);
/// });
/// ```
pub fn fetch_repositories(registry_url: String, tx: Sender<Message>) {
    let result = (|| -> Result<Vec<String>> {
        // Connect to registry
        let mut rex = Rex::connect(&registry_url)?;

        // Fetch repository list (already returns Vec<String>)
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
/// * `tx` - The channel sender for sending the result back to the UI thread
///
/// # Examples
///
/// ```no_run
/// use std::sync::mpsc::channel;
/// use rex::tui::worker::fetch_tags;
///
/// let (tx, rx) = channel();
/// let repo = "alpine".to_string();
/// std::thread::spawn(move || {
///     fetch_tags("localhost:5000".to_string(), repo, tx);
/// });
/// ```
#[allow(dead_code)] // TODO: Remove when integrated with views
pub fn fetch_tags(registry_url: String, repository: String, tx: Sender<Message>) {
    let repo_clone = repository.clone();

    let result = (|| -> Result<Vec<String>> {
        // Connect to registry
        let mut rex = Rex::connect(&registry_url)?;

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
/// * `tx` - The channel sender for sending the result back to the UI thread
///
/// # Examples
///
/// ```no_run
/// use std::sync::mpsc::channel;
/// use rex::tui::worker::fetch_manifest;
///
/// let (tx, rx) = channel();
/// let repo = "alpine".to_string();
/// let tag = "latest".to_string();
/// std::thread::spawn(move || {
///     fetch_manifest("localhost:5000".to_string(), repo, tag, tx);
/// });
/// ```
#[allow(dead_code)] // TODO: Remove when integrated with views
pub fn fetch_manifest(registry_url: String, repository: String, tag: String, tx: Sender<Message>) {
    let repo_clone = repository.clone();
    let tag_clone = tag.clone();

    let result = (|| -> Result<Vec<u8>> {
        // Connect to registry
        let mut rex = Rex::connect(&registry_url)?;

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
