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

    // Get registry URL from config
    let registry_url = match get_registry_url() {
        Ok(url) => url,
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    };

    // List images
    let images = match list_images(ctx, &registry_url, filter, limit) {
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

#[cfg(test)]
#[path = "handlers_tests.rs"]
mod tests;
