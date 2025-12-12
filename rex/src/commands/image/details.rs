use super::*;
use crate::context::VerbosityLevel;
use crate::format::{self, OutputFormat};

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
    }
}
