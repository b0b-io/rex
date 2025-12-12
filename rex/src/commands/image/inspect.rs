use super::*;
use crate::context::VerbosityLevel;
use crate::format::{self, OutputFormat};

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
    }
}
