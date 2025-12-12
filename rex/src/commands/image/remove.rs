use super::*;
use crate::config;
use crate::context::VerbosityLevel;
use crate::format;

/// Handle the image remove command
pub fn handle_image_remove(
    ctx: &crate::context::AppContext,
    reference: &str,
    force: bool,
    older_than: Option<u64>,
    dry_run: bool,
) {
    // Parse the reference to determine if it's a single tag or all tags
    let has_tag = reference.contains(':') || reference.contains('@');

    // Validate that --older-than is only used with repository names, not specific tags
    if older_than.is_some() && has_tag {
        format::error(
            ctx,
            "The --older-than flag can only be used with repository names (not specific tags or digests)",
        );
        std::process::exit(1);
    }

    // Get registry URL
    let registry_url = match get_registry_url() {
        Ok(url) => url,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // Build Rex instance
    let cache_dir = match get_registry_cache_dir(&registry_url) {
        Ok(dir) => dir,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    let creds_path = config::get_credentials_path();
    let credentials = if creds_path.exists() {
        if let Ok(store) = librex::auth::FileCredentialStore::new(creds_path) {
            store.get(&registry_url).ok().flatten()
        } else {
            None
        }
    } else {
        None
    };

    let mut builder = librex::Rex::builder()
        .registry_url(&registry_url)
        .with_cache(&cache_dir);

    if let Some(ref creds) = credentials {
        builder = builder.with_credentials(creds.clone());
    }

    let mut rex = match builder.build() {
        Ok(r) => r,
        Err(e) => {
            format::error(ctx, &format!("Failed to connect to registry: {}", e));
            std::process::exit(1);
        }
    };

    if has_tag {
        // Single tag deletion
        handle_single_tag_deletion(ctx, &mut rex, reference, force);
    } else {
        // Repository-level deletion (all tags or age-filtered)
        handle_repository_deletion(
            ctx,
            rex,
            reference,
            &registry_url,
            cache_dir,
            credentials,
            force,
            older_than,
            dry_run,
        );
    }
}

/// Handle single tag deletion
fn handle_single_tag_deletion(
    ctx: &crate::context::AppContext,
    rex: &mut librex::Rex,
    reference: &str,
    force: bool,
) {
    if !force {
        // Prompt for confirmation
        print!("Delete image '{}'? [y/N]: ", reference);
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return;
        }
    }

    format::print(
        ctx,
        VerbosityLevel::Verbose,
        &format!("Deleting image '{}'...", reference),
    );

    match rex.delete_tag(reference) {
        Ok(()) => {
            format::success(ctx, &format!("Deleted image '{}'", reference));
        }
        Err(e) => {
            format::error(ctx, &format!("Failed to delete image: {}", e));
            std::process::exit(1);
        }
    }
}

/// Handle repository-level deletion (all tags or age-filtered)
#[allow(clippy::too_many_arguments)]
fn handle_repository_deletion(
    ctx: &crate::context::AppContext,
    mut rex: librex::Rex,
    repository: &str,
    registry_url: &str,
    cache_dir: std::path::PathBuf,
    credentials: Option<librex::auth::Credentials>,
    force: bool,
    older_than: Option<u64>,
    dry_run: bool,
) {
    format::print(
        ctx,
        VerbosityLevel::Verbose,
        &format!("Listing tags for repository '{}'...", repository),
    );

    // Fetch tag metadata (with timestamps if older_than specified)
    let tag_infos = if older_than.is_some() {
        match fetch_tag_metadata_for_deletion(
            ctx,
            registry_url,
            repository,
            &cache_dir,
            credentials.clone(),
        ) {
            Ok(infos) => infos,
            Err(e) => {
                format::error(ctx, &e);
                std::process::exit(1);
            }
        }
    } else if dry_run {
        // For dry-run without age filtering, we still want to show full info
        match fetch_tag_metadata_for_deletion(
            ctx,
            registry_url,
            repository,
            &cache_dir,
            credentials.clone(),
        ) {
            Ok(infos) => infos,
            Err(e) => {
                format::error(ctx, &e);
                std::process::exit(1);
            }
        }
    } else {
        // Fast path: just need tag names
        let tags = match rex.list_tags(repository) {
            Ok(t) => t,
            Err(e) => {
                format::error(ctx, &format!("Failed to list tags: {}", e));
                std::process::exit(1);
            }
        };

        if tags.is_empty() {
            println!("No tags found for repository '{}'.", repository);
            return;
        }

        // Convert to TagInfo stubs (no metadata needed for delete-all without dry-run)
        tags.into_iter()
            .map(|tag| crate::image::TagInfo::new(tag, "N/A".to_string(), 0, None, vec![]))
            .collect()
    };

    if tag_infos.is_empty() {
        println!("No tags found for repository '{}'.", repository);
        return;
    }

    // Filter by age if requested
    let tags_to_delete = if let Some(days) = older_than {
        let filtered = filter_tags_by_age(&tag_infos, days);
        if filtered.is_empty() {
            println!("No tags older than {} days found.", days);
            return;
        }
        filtered
    } else {
        tag_infos
    };

    // Handle dry-run mode
    if dry_run {
        display_dry_run_results(repository, &tags_to_delete, older_than);
        return;
    }

    // Confirm (unless --force)
    if !force {
        display_deletion_preview(repository, &tags_to_delete, older_than);

        let prompt = if let Some(days) = older_than {
            format!(
                "Delete {} tags older than {} days? [y/N]: ",
                tags_to_delete.len(),
                days
            )
        } else {
            format!("Delete {} tags? [y/N]: ", tags_to_delete.len())
        };

        print!("{}", prompt);
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return;
        }
    }

    // Delete tags individually using existing infrastructure
    format::print(
        ctx,
        VerbosityLevel::Verbose,
        &format!(
            "Deleting {} tags from '{}'...",
            tags_to_delete.len(),
            repository
        ),
    );

    let mut deleted = Vec::new();
    let mut failed = Vec::new();

    for tag_info in &tags_to_delete {
        let ref_str = format!("{}:{}", repository, tag_info.tag);
        match rex.delete_tag(&ref_str) {
            Ok(()) => deleted.push(tag_info.tag.clone()),
            Err(e) => failed.push((tag_info.tag.clone(), e.to_string())),
        }
    }

    // Report results
    if !deleted.is_empty() {
        format::success(
            ctx,
            &format!("Deleted {} tags from '{}'", deleted.len(), repository),
        );

        if ctx.verbosity >= VerbosityLevel::Verbose {
            for tag in deleted {
                println!("  ✓ {}", tag);
            }
        }
    }

    if !failed.is_empty() {
        eprintln!("Failed to delete {} tags:", failed.len());
        for (tag, err) in failed {
            eprintln!("  ✗ {}: {}", tag, err);
        }
        std::process::exit(1);
    }
}

/// Display struct for dry-run results table
#[derive(tabled::Tabled)]
struct DryRunTagDisplay {
    #[tabled(rename = "TAG")]
    tag: String,
    #[tabled(rename = "AGE")]
    age: String,
    #[tabled(rename = "SIZE")]
    size: String,
    #[tabled(rename = "PLATFORM")]
    platforms: String,
}

/// Display dry-run results showing what would be deleted.
fn display_dry_run_results(
    repository: &str,
    tags_to_delete: &[crate::image::TagInfo],
    older_than: Option<u64>,
) {
    let header = if let Some(days) = older_than {
        format!(
            "The following {} tags from '{}' are older than {} days and would be deleted:",
            tags_to_delete.len(),
            repository,
            days
        )
    } else {
        format!(
            "The following {} tags from '{}' would be deleted:",
            tags_to_delete.len(),
            repository
        )
    };

    println!("{}", header);
    println!();

    // Use table format for detailed view
    use tabled::{Table, settings::Style};

    // Create display-friendly table rows
    let display_tags: Vec<DryRunTagDisplay> = tags_to_delete
        .iter()
        .map(|ti| DryRunTagDisplay {
            tag: ti.tag.clone(),
            age: ti.created.clone(),
            size: ti.size.clone(),
            platforms: ti.platforms.clone(),
        })
        .collect();

    let table = Table::new(&display_tags).with(Style::empty()).to_string();
    println!("{}", table);

    println!();
    println!(
        "Total: {} tags (dry-run mode, nothing deleted)",
        tags_to_delete.len()
    );
}

/// Display preview of tags to be deleted (before confirmation prompt).
fn display_deletion_preview(
    repository: &str,
    tags_to_delete: &[crate::image::TagInfo],
    older_than: Option<u64>,
) {
    let header = if let Some(days) = older_than {
        format!(
            "The following {} tags from '{}' are older than {} days and will be deleted:",
            tags_to_delete.len(),
            repository,
            days
        )
    } else {
        format!("The following tags from '{}' will be deleted:", repository)
    };

    println!("{}", header);

    for tag_info in tags_to_delete {
        let age_info = if !tag_info.created.is_empty() && tag_info.created != "N/A" {
            format!(" ({})", tag_info.created)
        } else {
            String::new()
        };
        println!("  - {}{}", tag_info.tag, age_info);
    }
    println!();
}

/// Fetch tag metadata with full timestamp resolution for deletion purposes.
///
/// For multi-platform images, this fetches each platform's manifest to determine
/// the creation timestamp, then uses the newest timestamp across all platforms.
fn fetch_tag_metadata_for_deletion(
    ctx: &crate::context::AppContext,
    registry_url: &str,
    repository: &str,
    cache_dir: &std::path::Path,
    credentials: Option<librex::auth::Credentials>,
) -> Result<Vec<crate::image::TagInfo>, String> {
    use crate::image::TagMetadataFetcher;

    let fetcher = TagMetadataFetcher::new(
        registry_url.to_string(),
        cache_dir,
        credentials.clone(),
        8, // Default concurrency for metadata fetching
    );

    format::print(
        ctx,
        VerbosityLevel::Verbose,
        "Fetching tag metadata with timestamps...",
    );

    let mut tag_infos = fetcher
        .fetch_tags(repository)
        .map_err(|e| format!("Failed to fetch tag metadata: {}", e))?;

    // Identify tags that need timestamp resolution (multi-platform images without timestamps)
    let needs_resolution: Vec<usize> = tag_infos
        .iter()
        .enumerate()
        .filter_map(|(idx, ti)| {
            // If no timestamp and has multiple platforms, need to resolve
            if ti.created_timestamp.is_none() && !ti.platforms.is_empty() && ti.platforms != "N/A" {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    if !needs_resolution.is_empty() {
        format::print(
            ctx,
            VerbosityLevel::Verbose,
            &format!(
                "Resolving timestamps for {} multi-platform images...",
                needs_resolution.len()
            ),
        );

        // Build Rex client for multi-platform timestamp resolution
        let mut builder = librex::Rex::builder()
            .registry_url(registry_url)
            .with_cache(cache_dir);

        if let Some(ref creds) = credentials {
            builder = builder.with_credentials(creds.clone());
        }

        let mut rex = builder
            .build()
            .map_err(|e| format!("Failed to connect to registry: {}", e))?;

        // Resolve timestamps for multi-platform images
        for idx in needs_resolution {
            let tag_info = &mut tag_infos[idx];
            let reference = format!("{}:{}", repository, tag_info.tag);

            if let Ok(Some(timestamp)) = resolve_multiplatform_timestamp(
                &mut rex,
                &reference,
                repository,
                registry_url,
                cache_dir,
                credentials.as_ref(),
            ) {
                tag_info.created_timestamp = Some(timestamp);
                tag_info.created = librex::format::format_timestamp(&timestamp);
            }
        }
    }

    Ok(tag_infos)
}

/// Resolve the newest timestamp across all platforms in a multi-platform image.
///
/// For deletion decisions, we use the NEWEST timestamp across all platforms.
/// This ensures we only delete an image if ALL platforms are older than the threshold.
fn resolve_multiplatform_timestamp(
    rex: &mut librex::Rex,
    reference: &str,
    repository: &str,
    registry_url: &str,
    cache_dir: &std::path::Path,
    credentials: Option<&librex::auth::Credentials>,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, String> {
    // Fetch the manifest to check if it's multi-platform
    let (manifest_or_index, _digest) = rex
        .get_manifest(reference)
        .map_err(|e| format!("Failed to fetch manifest: {}", e))?;

    let index = match manifest_or_index {
        librex::oci::ManifestOrIndex::Index(idx) => idx,
        librex::oci::ManifestOrIndex::Manifest(_) => {
            // Single-platform image, timestamp should already be resolved
            return Ok(None);
        }
    };

    // Extract platform descriptors
    let platform_descriptors: Vec<_> = index.manifests().to_vec();

    if platform_descriptors.is_empty() {
        return Ok(None);
    }

    // Fetch timestamps from each platform in parallel using rayon
    use rayon::prelude::*;

    let concurrency = platform_descriptors.len().clamp(1, 8);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()
        .map_err(|e| format!("Failed to create thread pool: {}", e))?;

    let registry_url = registry_url.to_string();
    let cache_dir = cache_dir.to_path_buf();
    let credentials = credentials.cloned();
    let repository = repository.to_string();

    let timestamps: Vec<chrono::DateTime<chrono::Utc>> = pool.install(|| {
        platform_descriptors
            .par_iter()
            .filter_map(|desc| {
                fetch_platform_timestamp(
                    &registry_url,
                    &repository,
                    desc,
                    Some(&cache_dir),
                    credentials.clone(),
                )
            })
            .collect()
    });

    // Return the NEWEST timestamp across all platforms
    Ok(timestamps.into_iter().max())
}

/// Fetch the creation timestamp for a specific platform manifest.
fn fetch_platform_timestamp(
    registry_url: &str,
    repository: &str,
    platform_desc: &librex::oci::Descriptor,
    cache_dir: Option<&std::path::Path>,
    credentials: Option<librex::auth::Credentials>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    use std::str::FromStr;

    // Build per-thread Rex client
    let mut builder = librex::Rex::builder().registry_url(registry_url);

    if let Some(dir) = cache_dir {
        builder = builder.with_cache(dir);
    }

    if let Some(ref creds) = credentials {
        builder = builder.with_credentials(creds.clone());
    }

    let mut thread_rex = builder.build().ok()?;

    // Fetch platform-specific manifest by digest
    let platform_digest = platform_desc.digest().to_string();
    let platform_ref = format!("{}@{}", repository, platform_digest);

    let (manifest_or_index, _) = thread_rex.get_manifest(&platform_ref).ok()?;

    let manifest = match manifest_or_index {
        librex::oci::ManifestOrIndex::Manifest(m) => m,
        librex::oci::ManifestOrIndex::Index(_) => return None, // Shouldn't happen
    };

    // Get config blob to extract timestamp
    let config_digest_str = manifest.config().digest().to_string();
    let config_digest = librex::digest::Digest::from_str(&config_digest_str).ok()?;

    let config_bytes = thread_rex.get_blob(repository, &config_digest).ok()?;

    let config: librex::oci::ImageConfiguration = serde_json::from_slice(&config_bytes).ok()?;

    // Parse timestamp
    config.created().as_ref().and_then(|ts| {
        chrono::DateTime::parse_from_rfc3339(ts)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    })
}

/// Filter tags by age threshold.
///
/// Only includes tags where the creation timestamp is older than the specified number of days.
/// Tags without timestamps are skipped with a warning printed to stderr.
///
/// # Arguments
///
/// * `tag_infos` - Slice of TagInfo structs with timestamps
/// * `days` - Age threshold in days
///
/// # Returns
///
/// Vector of TagInfo for tags that are older than the threshold
fn filter_tags_by_age(
    tag_infos: &[crate::image::TagInfo],
    days: u64,
) -> Vec<crate::image::TagInfo> {
    use chrono::Utc;

    let threshold = Utc::now() - chrono::Duration::days(days as i64);

    tag_infos
        .iter()
        .filter(|tag_info| {
            match tag_info.created_timestamp {
                Some(ts) => ts < threshold,
                None => {
                    // No timestamp available - skip this tag for safety
                    eprintln!("Warning: Tag '{}' has no timestamp, skipping", tag_info.tag);
                    false
                }
            }
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for age-based deletion functionality

    #[test]
    fn test_filter_tags_by_age_basic() {
        use chrono::{Duration, Utc};

        let now = Utc::now();
        let old_timestamp = now - Duration::days(100);
        let recent_timestamp = now - Duration::days(5);

        let tags = vec![
            crate::image::TagInfo::new(
                "old-tag".to_string(),
                "sha256:abc123".to_string(),
                1000,
                Some(old_timestamp),
                vec!["linux/amd64".to_string()],
            ),
            crate::image::TagInfo::new(
                "recent-tag".to_string(),
                "sha256:def456".to_string(),
                2000,
                Some(recent_timestamp),
                vec!["linux/amd64".to_string()],
            ),
        ];

        let filtered = filter_tags_by_age(&tags, 30);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].tag, "old-tag");
    }

    #[test]
    fn test_filter_tags_by_age_no_matches() {
        use chrono::{Duration, Utc};

        let now = Utc::now();
        let recent_timestamp = now - Duration::days(5);

        let tags = vec![
            crate::image::TagInfo::new(
                "recent1".to_string(),
                "sha256:abc123".to_string(),
                1000,
                Some(recent_timestamp),
                vec!["linux/amd64".to_string()],
            ),
            crate::image::TagInfo::new(
                "recent2".to_string(),
                "sha256:def456".to_string(),
                2000,
                Some(recent_timestamp),
                vec!["linux/amd64".to_string()],
            ),
        ];

        let filtered = filter_tags_by_age(&tags, 30);

        assert_eq!(filtered.len(), 0, "Expected no tags older than 30 days");
    }

    #[test]
    fn test_filter_tags_missing_timestamps() {
        use chrono::{Duration, Utc};

        let now = Utc::now();
        let old_timestamp = now - Duration::days(100);

        let tags = vec![
            crate::image::TagInfo::new(
                "old-tag".to_string(),
                "sha256:abc123".to_string(),
                1000,
                Some(old_timestamp),
                vec!["linux/amd64".to_string()],
            ),
            crate::image::TagInfo::new(
                "no-timestamp".to_string(),
                "sha256:ghi789".to_string(),
                3000,
                None, // No timestamp
                vec!["linux/amd64".to_string()],
            ),
        ];

        let filtered = filter_tags_by_age(&tags, 30);

        // Should only include the old tag with timestamp, skip the one without
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].tag, "old-tag");
    }

    #[test]
    fn test_filter_tags_by_age_boundary() {
        use chrono::{Duration, Utc};

        let now = Utc::now();
        // Exactly 30 days old
        let boundary_timestamp = now - Duration::days(30);
        // 30 days + 1 hour old (should be filtered)
        let just_over_threshold = now - Duration::days(30) - Duration::hours(1);
        // 30 days - 1 hour old (should NOT be filtered)
        let just_under_threshold = now - Duration::days(30) + Duration::hours(1);

        let tags = vec![
            crate::image::TagInfo::new(
                "exactly-30-days".to_string(),
                "sha256:aaa".to_string(),
                1000,
                Some(boundary_timestamp),
                vec!["linux/amd64".to_string()],
            ),
            crate::image::TagInfo::new(
                "over-30-days".to_string(),
                "sha256:bbb".to_string(),
                1000,
                Some(just_over_threshold),
                vec!["linux/amd64".to_string()],
            ),
            crate::image::TagInfo::new(
                "under-30-days".to_string(),
                "sha256:ccc".to_string(),
                1000,
                Some(just_under_threshold),
                vec!["linux/amd64".to_string()],
            ),
        ];

        let filtered = filter_tags_by_age(&tags, 30);

        // Should include the exactly 30 days and over 30 days tags
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|t| t.tag == "exactly-30-days"));
        assert!(filtered.iter().any(|t| t.tag == "over-30-days"));
        assert!(!filtered.iter().any(|t| t.tag == "under-30-days"));
    }
}
