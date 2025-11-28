use super::*;
use crate::context::VerbosityLevel;
use crate::format::{self, OutputFormat};

/// Handle the image list command
pub fn handle_image_list(
    ctx: &crate::context::AppContext,
    format: OutputFormat,
    quiet: bool,
    filter: Option<&str>,
    limit: Option<usize>,
) {
    format::print(
        ctx,
        VerbosityLevel::Verbose,
        "Listing images from registry...",
    );
    if let Some(pattern) = filter {
        format::print(
            ctx,
            VerbosityLevel::Verbose,
            &format!("Applying filter: {}", pattern),
        );
    }

    // Get registry entry from config
    let registry_entry = match get_registry_entry() {
        Ok(entry) => entry,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // List images
    let images = match list_images(
        ctx,
        &registry_entry.url,
        registry_entry.dockerhub_compat,
        filter,
        limit,
    ) {
        Ok(imgs) => imgs,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // Handle quiet mode
    if quiet {
        for image in images {
            println!("{}", image.name);
        }
        return;
    }

    // Handle empty results
    if images.is_empty() {
        println!("No images found.");
        return;
    }

    // Format output
    match format {
        OutputFormat::Pretty => {
            use tabled::{Table, settings::Style};
            let table = Table::new(&images).with(Style::empty()).to_string();
            println!("{}", table);
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&images) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error formatting JSON: {}", e);
                std::process::exit(1);
            }
        },
        OutputFormat::Yaml => match serde_yaml::to_string(&images) {
            Ok(yaml) => print!("{}", yaml),
            Err(e) => {
                eprintln!("Error formatting YAML: {}", e);
                std::process::exit(1);
            }
        },
    }
}

/// Handle the image tags command (list tags for a specific image)
pub fn handle_image_tags(
    ctx: &crate::context::AppContext,
    image_name: &str,
    format: OutputFormat,
    quiet: bool,
    filter: Option<&str>,
    limit: Option<usize>,
) {
    format::print(
        ctx,
        VerbosityLevel::Verbose,
        &format!("Listing tags for image: {}", image_name),
    );
    if let Some(pattern) = filter {
        format::print(
            ctx,
            VerbosityLevel::Verbose,
            &format!("Applying filter: {}", pattern),
        );
    }

    // Get registry URL from config
    let registry_url = match get_registry_url() {
        Ok(url) => url,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // List tags for the image
    let tags = match list_tags(ctx, &registry_url, image_name, filter, limit) {
        Ok(tags) => tags,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // Handle quiet mode
    if quiet {
        for tag in tags {
            println!("{}", tag.tag);
        }
        return;
    }

    // Handle empty results
    if tags.is_empty() {
        println!("No tags found for image '{}'.", image_name);
        return;
    }

    // Format output
    match format {
        OutputFormat::Pretty => {
            use tabled::{Table, settings::Style};
            let table = Table::new(&tags).with(Style::empty()).to_string();
            println!("{}", table);
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&tags) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error formatting JSON: {}", e);
                std::process::exit(1);
            }
        },
        OutputFormat::Yaml => match serde_yaml::to_string(&tags) {
            Ok(yaml) => print!("{}", yaml),
            Err(e) => {
                eprintln!("Error formatting YAML: {}", e);
                std::process::exit(1);
            }
        },
    }
}

/// Handle the image details command (show details for image:tag or image@digest)
pub fn handle_image_details(
    ctx: &crate::context::AppContext,
    reference: &str,
    format: OutputFormat,
) {
    format::print(
        ctx,
        VerbosityLevel::Verbose,
        &format!("Fetching details for: {}", reference),
    );

    // Get registry URL from config
    let registry_url = match get_registry_url() {
        Ok(url) => url,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // Get image details
    let details = match get_image_details(&registry_url, reference) {
        Ok(details) => details,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // Format output
    match format {
        OutputFormat::Pretty => {
            println!("{}", details.format_pretty());
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&details) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error formatting JSON: {}", e);
                std::process::exit(1);
            }
        },
        OutputFormat::Yaml => match serde_yaml::to_string(&details) {
            Ok(yaml) => print!("{}", yaml),
            Err(e) => {
                eprintln!("Error formatting YAML: {}", e);
                std::process::exit(1);
            }
        },
    }
}

/// Handle the image inspect command (full detailed inspection)
pub fn handle_image_inspect(
    ctx: &crate::context::AppContext,
    reference: &str,
    format: OutputFormat,
    platform: Option<&str>,
    raw_manifest: bool,
    raw_config: bool,
) {
    format::print(
        ctx,
        VerbosityLevel::Verbose,
        &format!("Inspecting image: {}", reference),
    );

    // Get registry URL from config
    let registry_url = match get_registry_url() {
        Ok(url) => url,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // Get full inspection details
    let inspect =
        match get_image_inspect(&registry_url, reference, platform, raw_manifest, raw_config) {
            Ok(inspect) => inspect,
            Err(e) => {
                format::error(ctx, &e);
                std::process::exit(1);
            }
        };

    // Handle raw output flags (these take precedence over format flags)
    if raw_manifest && inspect.raw_manifest.is_some() {
        println!("{}", inspect.raw_manifest.as_ref().unwrap());
        return;
    }

    if raw_config && inspect.raw_config.is_some() {
        println!("{}", inspect.raw_config.as_ref().unwrap());
        return;
    }

    // Format output
    match format {
        OutputFormat::Pretty => {
            println!("{}", inspect.format_pretty());
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&inspect) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error formatting JSON: {}", e);
                std::process::exit(1);
            }
        },
        OutputFormat::Yaml => match serde_yaml::to_string(&inspect) {
            Ok(yaml) => print!("{}", yaml),
            Err(e) => {
                eprintln!("Error formatting YAML: {}", e);
                std::process::exit(1);
            }
        },
    }
}

/// Handle the image remove command
pub fn handle_image_remove(ctx: &crate::context::AppContext, reference: &str, force: bool) {
    // Parse the reference to determine if it's a single tag or all tags
    let has_tag = reference.contains(':') || reference.contains('@');

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
        .with_cache(cache_dir);

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
    } else {
        // All tags deletion
        let repo = reference;

        // First, list tags to show what will be deleted
        format::print(
            ctx,
            VerbosityLevel::Verbose,
            &format!("Listing tags for repository '{}'...", repo),
        );

        let tags = match rex.list_tags(repo) {
            Ok(t) => t,
            Err(e) => {
                format::error(ctx, &format!("Failed to list tags: {}", e));
                std::process::exit(1);
            }
        };

        if tags.is_empty() {
            println!("No tags found for repository '{}'.", repo);
            return;
        }

        if !force {
            // Show what will be deleted and confirm
            println!("The following tags will be deleted from '{}':", repo);
            for tag in &tags {
                println!("  - {}", tag);
            }
            print!("\nDelete {} tags? [y/N]: ", tags.len());

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
            &format!("Deleting {} tags from '{}'...", tags.len(), repo),
        );

        match rex.delete_all_tags(repo) {
            Ok(deleted) => {
                format::success(
                    ctx,
                    &format!("Deleted {} tags from '{}'", deleted.len(), repo),
                );

                // Show deleted tags in verbose mode
                if ctx.verbosity >= VerbosityLevel::Verbose {
                    for tag in deleted {
                        println!("  ✓ {}", tag);
                    }
                }
            }
            Err(e) => {
                format::error(ctx, &format!("Failed to delete tags: {}", e));
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
#[path = "handlers_tests.rs"]
mod tests;
