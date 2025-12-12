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
    }
}
