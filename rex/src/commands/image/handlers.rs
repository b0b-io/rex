use super::*;
use crate::format::{self, OutputFormat};

/// Handle the image list command
pub async fn handle_image_list(
    ctx: &crate::context::AppContext,
    format: OutputFormat,
    quiet: bool,
    filter: Option<&str>,
    limit: Option<usize>,
) {
    // Get registry URL from config
    let registry_url = match get_registry_url() {
        Ok(url) => url,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // List images
    let images = match list_images(ctx, &registry_url, filter, limit).await {
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
            use tabled::Table;
            let table = Table::new(&images).to_string();
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
pub async fn handle_image_tags(
    ctx: &crate::context::AppContext,
    image_name: &str,
    format: OutputFormat,
    quiet: bool,
    filter: Option<&str>,
    limit: Option<usize>,
) {
    // Get registry URL from config
    let registry_url = match get_registry_url() {
        Ok(url) => url,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // List tags for the image
    let tags = match list_tags(&registry_url, image_name, filter, limit).await {
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
            use tabled::Table;
            let table = Table::new(&tags).to_string();
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
pub async fn handle_image_details(
    ctx: &crate::context::AppContext,
    reference: &str,
    format: OutputFormat,
) {
    // Get registry URL from config
    let registry_url = match get_registry_url() {
        Ok(url) => url,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // Get image details
    let details = match get_image_details(&registry_url, reference).await {
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
pub async fn handle_image_inspect(
    ctx: &crate::context::AppContext,
    reference: &str,
    format: OutputFormat,
    _platform: Option<&str>,
    _raw_manifest: bool,
    _raw_config: bool,
) {
    // Get registry URL from config
    let registry_url = match get_registry_url() {
        Ok(url) => url,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // Get full inspection details
    let inspect = match get_image_inspect(&registry_url, reference).await {
        Ok(inspect) => inspect,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // TODO: Implement --raw-manifest and --raw-config flags
    // TODO: Implement --platform flag for multi-arch images

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

#[cfg(test)]
#[path = "handlers_tests.rs"]
mod tests;
