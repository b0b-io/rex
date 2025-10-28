use crate::config;
use crate::context::VerbosityLevel;
use crate::format::{self, Formattable};
use librex::auth::CredentialStore;
use serde::Serialize;

pub mod handlers;

/// Search results containing both images and tags
#[derive(Debug, Serialize)]
pub struct SearchResults {
    pub query: String,
    pub images: ImageResults,
    pub tags: TagResults,
}

/// Image search results
#[derive(Debug, Serialize)]
pub struct ImageResults {
    pub total_results: usize,
    pub results: Vec<ImageResult>,
}

/// Tag search results
#[derive(Debug, Serialize)]
pub struct TagResults {
    pub total_results: usize,
    pub results: Vec<TagResult>,
}

/// Single image search result
#[derive(Debug, Serialize)]
pub struct ImageResult {
    pub name: String,
}

/// Single tag search result
#[derive(Debug, Serialize)]
pub struct TagResult {
    pub image: String,
    pub tag: String,
    pub reference: String,
}

impl Formattable for SearchResults {
    fn format_pretty(&self) -> String {
        let mut output = String::new();

        // Images section
        if !self.images.results.is_empty() {
            output.push_str("Images:\n");
            for result in &self.images.results {
                output.push_str(&format!("  {}\n", result.name));
            }
        }

        // Tags section
        if !self.tags.results.is_empty() {
            if !self.images.results.is_empty() {
                output.push('\n');
            }
            output.push_str("Tags:\n");
            for result in &self.tags.results {
                output.push_str(&format!("  {}\n", result.reference));
            }
        }

        // If no results
        if self.images.results.is_empty() && self.tags.results.is_empty() {
            output.push_str("No results found\n");
        }

        output
    }
}

/// Perform unified search across images and tags
pub async fn search(
    ctx: &crate::context::AppContext,
    query: &str,
    limit: Option<usize>,
) -> Result<SearchResults, String> {
    // Get the default registry from context config
    let registry = if let Some(default_name) = &ctx.config.registries.default {
        ctx.config
            .registries
            .list
            .iter()
            .find(|r| r.name == *default_name)
            .ok_or_else(|| format!("Default registry '{}' not found", default_name))?
    } else {
        return Err(crate::commands::registry::no_default_registry_error(
            &ctx.config.registries.list,
        ));
    };

    format::print(
        ctx,
        VerbosityLevel::VeryVerbose,
        &format!("Using registry: {} ({})", registry.name, registry.url),
    );

    // Get cache directory
    let cache_dir = config::get_registry_cache_dir(&registry.url)?;

    // Load credentials if available
    let creds_path = config::get_credentials_path();
    let credentials = if creds_path.exists() {
        if let Ok(store) = librex::auth::FileCredentialStore::new(creds_path) {
            store.get(&registry.url).ok().flatten()
        } else {
            None
        }
    } else {
        None
    };

    // Build Rex client
    let mut builder = librex::Rex::builder()
        .registry_url(&registry.url)
        .with_cache(cache_dir.as_path());

    if let Some(creds) = credentials {
        builder = builder.with_credentials(creds);
    }

    let mut rex = builder
        .build()
        .await
        .map_err(|e| format!("Failed to connect to registry: {}", e))?;

    let formatter = crate::format::create_formatter(ctx);

    // Search images (repositories)
    let spinner = formatter.spinner("Searching repositories...");
    let image_results_res = rex.search_repositories(query).await;
    let mut image_results = match image_results_res {
        Ok(results) => {
            formatter.finish_progress(spinner, &format!("Found {} matching images", results.len()));
            results
        }
        Err(e) => {
            spinner.finish_and_clear();
            return Err(format!("Failed to search repositories: {}", e));
        }
    };

    // Apply limit if specified
    if let Some(limit) = limit {
        image_results.truncate(limit);
    }

    // Search tags across all images
    let spinner = formatter.spinner("Searching tags...");
    let tag_results_res = rex.search_images(query).await;
    let mut tag_results = match tag_results_res {
        Ok(results) => {
            formatter.finish_progress(spinner, &format!("Found {} matching tags", results.len()));
            results
        }
        Err(e) => {
            spinner.finish_and_clear();
            return Err(format!("Failed to search tags: {}", e));
        }
    };

    // Apply limit if specified
    if let Some(limit) = limit {
        tag_results.truncate(limit);
    }

    // Convert to our result structures
    let images = ImageResults {
        total_results: image_results.len(),
        results: image_results
            .into_iter()
            .map(|result| ImageResult { name: result.value })
            .collect(),
    };

    let tags = TagResults {
        total_results: tag_results.len(),
        results: tag_results
            .into_iter()
            .map(|result| {
                // Parse the "repo:tag" format from the value
                let parts: Vec<&str> = result.value.split(':').collect();
                let (image, tag) = if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    // Fallback in case format is unexpected
                    (result.value.clone(), String::new())
                };

                TagResult {
                    reference: result.value,
                    image,
                    tag,
                }
            })
            .collect(),
    };

    Ok(SearchResults {
        query: query.to_string(),
        images,
        tags,
    })
}
