use crate::config;
use crate::format::{self, Formattable, OutputFormat};

/// Implement Formattable for Config to enable output formatting
impl Formattable for config::Config {
    fn format_pretty(&self) -> String {
        let mut output = format!(
            "[style]\nformat = \"{}\"\ncolor = {}\n",
            match self.style.format {
                OutputFormat::Pretty => "pretty",
                OutputFormat::Json => "json",
                OutputFormat::Yaml => "yaml",
            },
            self.style.color
        );

        // Add registries section
        output.push_str("\n[registries]\n");
        if let Some(ref default) = self.registries.default {
            output.push_str(&format!("default = \"{}\"\n", default));
        }

        if !self.registries.list.is_empty() {
            output.push('\n');
            for registry in &self.registries.list {
                output.push_str(&format!(
                    "[[registries.list]]\nname = \"{}\"\nurl = \"{}\"\n\n",
                    registry.name, registry.url
                ));
            }
        }

        output
    }
}

/// Handle the config init subcommand
pub fn handle_init(ctx: &crate::context::AppContext) {
    let config_path = config::get_config_path();
    match config::init_config(&config_path) {
        Ok(_) => {
            format::success(
                ctx,
                &format!("Initialized config file at: {}", config_path.display()),
            );
        }
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}

/// Handle the config get subcommand
pub fn handle_get(ctx: &crate::context::AppContext, key: Option<&str>, format: OutputFormat) {
    let config_path = config::get_config_path();

    match key {
        Some(k) => {
            // Get specific key
            match config::get_config_value(&config_path, k) {
                Ok(value) => println!("{}", value),
                Err(e) => {
                    format::error(ctx, &e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            // Display all config
            match config::display_config(&config_path) {
                Ok(cfg) => match crate::format::format_output(&cfg, format) {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        format::error(ctx, &format!("formatting output: {}", e));
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    format::error(ctx, &e);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Handle the config set subcommand
pub fn handle_set(ctx: &crate::context::AppContext, key: Option<&str>, value: Option<&str>) {
    let config_path = config::get_config_path();

    match (key, value) {
        (Some(k), Some(v)) => {
            // Set specific key
            match config::set_config_value(&config_path, k, v) {
                Ok(_) => format::success(ctx, &format!("Set {} = {}", k, v)),
                Err(e) => {
                    format::error(ctx, &e);
                    std::process::exit(1);
                }
            }
        }
        (None, None) => {
            // Open editor
            match config::edit_config(&config_path) {
                Ok(_) => {}
                Err(e) => {
                    format::error(ctx, &e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            format::error(
                ctx,
                "Invalid arguments. Use 'rex config set <key> <value>' or 'rex config set' to edit.",
            );
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
