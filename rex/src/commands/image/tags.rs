use super::*;
use crate::context::VerbosityLevel;
use crate::format::{self, OutputFormat};

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
    }
}
