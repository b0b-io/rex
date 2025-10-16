//! Search and filtering module.
//!
//! This module provides fuzzy search capabilities for repositories and tags,
//! using the nucleo-matcher library for fzf-like matching with scoring and ranking.

use nucleo_matcher::{Config, Matcher, pattern::Normalization, pattern::Pattern};
use std::collections::HashMap;

// Re-export CaseMatching for public API
pub use nucleo_matcher::pattern::CaseMatching;

#[cfg(test)]
mod tests;

/// A search result with relevance score.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SearchResult {
    /// The matched string.
    pub value: String,
    /// Relevance score (higher is better).
    pub score: u32,
}

impl SearchResult {
    /// Creates a new search result.
    pub fn new(value: String, score: u32) -> Self {
        Self { value, score }
    }
}

/// Performs fuzzy search on a list of targets.
///
/// Uses nucleo-matcher's fuzzy matching algorithm (same as fzf) where query
/// characters must appear in order in the target string but don't need to be contiguous.
///
/// # Arguments
///
/// * `query` - The search query string
/// * `targets` - List of strings to search through
/// * `case_matching` - Case sensitivity mode (Ignore, Smart, or Respect)
///
/// # Returns
///
/// A vector of search results sorted by relevance (highest score first).
///
/// # Examples
///
/// ```
/// use librex::search::{fuzzy_search, CaseMatching};
///
/// let targets = vec!["alpine".to_string(), "ubuntu".to_string(), "nginx".to_string()];
/// let results = fuzzy_search("alp", &targets, CaseMatching::Ignore);
/// assert!(results.iter().any(|r| r.value == "alpine"));
/// ```
pub fn fuzzy_search(
    query: &str,
    targets: &[String],
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    if query.is_empty() {
        // Empty query returns all targets with equal score
        return targets
            .iter()
            .map(|t| SearchResult::new(t.clone(), 0))
            .collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, case_matching, Normalization::Smart);

    // Use match_list for convenience - returns Vec<(String, u32)>
    let matches = pattern.match_list(targets.iter().map(|s| s.as_str()), &mut matcher);

    // Convert to SearchResult and sort by score descending
    let mut results: Vec<SearchResult> = matches
        .into_iter()
        .map(|(value, score)| SearchResult::new(value.to_string(), score))
        .collect();

    // Sort by score descending (highest score first), then alphabetically
    results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.value.cmp(&b.value)));
    results
}

/// Searches repositories by name.
///
/// # Arguments
///
/// * `query` - The search query
/// * `repositories` - List of repository names to search
///
/// # Returns
///
/// A vector of matching repository names sorted by relevance.
///
/// # Examples
///
/// ```
/// use librex::search::search_repositories;
///
/// let repos = vec!["alpine".to_string(), "ubuntu".to_string(), "nginx".to_string()];
/// let results = search_repositories("alp", &repos);
/// assert_eq!(results[0].value, "alpine");
/// ```
pub fn search_repositories(query: &str, repositories: &[String]) -> Vec<SearchResult> {
    fuzzy_search(query, repositories, CaseMatching::Smart)
}

/// Searches tags within a repository or across all repositories.
///
/// # Arguments
///
/// * `query` - The search query
/// * `tags` - List of tag names to search
///
/// # Returns
///
/// A vector of matching tags sorted by relevance.
///
/// # Examples
///
/// ```
/// use librex::search::search_tags;
///
/// let tags = vec!["latest".to_string(), "3.19".to_string(), "stable".to_string()];
/// let results = search_tags("lat", &tags);
/// assert_eq!(results[0].value, "latest");
/// ```
pub fn search_tags(query: &str, tags: &[String]) -> Vec<SearchResult> {
    fuzzy_search(query, tags, CaseMatching::Smart)
}

/// Searches for images (repository:tag combinations).
///
/// # Arguments
///
/// * `query` - The search query (can include ":" to search both repo and tag)
/// * `repositories` - List of repository names
/// * `tags_map` - Map of repository names to their tags
///
/// # Returns
///
/// A vector of matching image references (repository:tag) sorted by relevance.
///
/// # Examples
///
/// ```
/// use librex::search::search_images;
/// use std::collections::HashMap;
///
/// let repos = vec!["alpine".to_string(), "ubuntu".to_string()];
/// let mut tags_map = HashMap::new();
/// tags_map.insert("alpine".to_string(), vec!["latest".to_string(), "3.19".to_string()]);
/// tags_map.insert("ubuntu".to_string(), vec!["latest".to_string(), "22.04".to_string()]);
///
/// let results = search_images("alp:lat", &repos, &tags_map);
/// ```
pub fn search_images(
    query: &str,
    repositories: &[String],
    tags_map: &HashMap<String, Vec<String>>,
) -> Vec<SearchResult> {
    // Check if query contains ":"
    if let Some(colon_idx) = query.find(':') {
        // Split into repository and tag queries
        let repo_query = &query[..colon_idx];
        let tag_query = &query[colon_idx + 1..];

        // Search repositories first
        let repo_results = search_repositories(repo_query, repositories);

        // For each matching repository, search its tags
        let mut image_results = Vec::new();
        for repo_result in repo_results {
            if let Some(tags) = tags_map.get(&repo_result.value) {
                let tag_results = search_tags(tag_query, tags);
                for tag_result in tag_results {
                    let image = format!("{}:{}", repo_result.value, tag_result.value);
                    // Combined score: average of repo and tag scores
                    let combined_score = (repo_result.score + tag_result.score) / 2;
                    image_results.push(SearchResult::new(image, combined_score));
                }
            }
        }

        image_results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.value.cmp(&b.value)));
        image_results
    } else {
        // No ":", search only repository names and combine with all tags
        let repo_results = search_repositories(query, repositories);
        let mut image_results = Vec::new();

        for repo_result in repo_results {
            if let Some(tags) = tags_map.get(&repo_result.value) {
                for tag in tags {
                    let image = format!("{}:{}", repo_result.value, tag);
                    image_results.push(SearchResult::new(image, repo_result.score));
                }
            }
        }

        image_results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.value.cmp(&b.value)));
        image_results
    }
}
