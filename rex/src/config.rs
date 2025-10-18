use crate::output::{Formattable, OutputFormat};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Style configuration
    #[serde(default)]
    pub style: StyleConfig,
}

/// Style configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    /// Output format: pretty, json, yaml
    #[serde(default = "default_format")]
    pub format: OutputFormat,
    /// Enable color output
    #[serde(default = "default_color")]
    pub color: bool,
}

fn default_format() -> OutputFormat {
    OutputFormat::Pretty
}

fn default_color() -> bool {
    true
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Pretty,
            color: true,
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let contents =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

        toml::from_str(&contents).map_err(|e| format!("Failed to parse config file: {}", e))
    }

    /// Save configuration to a file
    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(path, toml_str).map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }
}

impl Formattable for Config {
    fn format_pretty(&self) -> String {
        format!(
            "[style]\nformat = \"{}\"\ncolor = {}",
            match self.style.format {
                OutputFormat::Pretty => "pretty",
                OutputFormat::Json => "json",
                OutputFormat::Yaml => "yaml",
            },
            self.style.color
        )
    }
}

/// Get the config file path, respecting REX_CONFIG environment variable
pub fn get_config_path() -> PathBuf {
    if let Ok(config_path) = env::var("REX_CONFIG") {
        return PathBuf::from(config_path);
    }

    // Default to ~/.config/rex/config.toml
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("rex").join("config.toml")
    } else {
        // Fallback to current directory
        PathBuf::from("config.toml")
    }
}

/// Initialize a new config file with default values
pub fn init_config(config_path: &PathBuf) -> Result<(), String> {
    if config_path.exists() {
        return Err(
            "Config file already exists. Use 'rex config set' to edit or 'rm' to recreate."
                .to_string(),
        );
    }

    let config = Config::default();
    config.save(config_path)?;

    Ok(())
}

/// Get a configuration value by key (supports nested keys like "style.format")
pub fn get_config_value(config_path: &PathBuf, key: &str) -> Result<String, String> {
    let config = Config::load(config_path)?;

    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["style", "format"] => Ok(match config.style.format {
            OutputFormat::Pretty => "pretty".to_string(),
            OutputFormat::Json => "json".to_string(),
            OutputFormat::Yaml => "yaml".to_string(),
        }),
        ["style", "color"] => Ok(config.style.color.to_string()),
        _ => Err(format!("Unknown config key: {}", key)),
    }
}

/// Set a configuration value by key (supports nested keys like "style.format")
pub fn set_config_value(config_path: &PathBuf, key: &str, value: &str) -> Result<(), String> {
    // Load existing config or create default
    let mut config = if config_path.exists() {
        Config::load(config_path)?
    } else {
        Config::default()
    };

    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["style", "format"] => {
            config.style.format = OutputFormat::from(value);
        }
        ["style", "color"] => {
            config.style.color = value.parse::<bool>().map_err(|_| {
                format!("Invalid boolean value '{}'. Use 'true' or 'false'.", value)
            })?;
        }
        _ => return Err(format!("Unknown config key: {}", key)),
    }

    config.save(config_path)?;

    Ok(())
}

/// Open the config file in the user's editor
pub fn edit_config(config_path: &PathBuf) -> Result<(), String> {
    // Get editor from environment
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .map_err(|_| "No editor found. Set EDITOR or VISUAL environment variable.".to_string())?;

    // Create config file if it doesn't exist
    if !config_path.exists() {
        let config = Config::default();
        config.save(config_path)?;
    }

    // Open editor
    let status = Command::new(editor)
        .arg(config_path)
        .status()
        .map_err(|e| format!("Failed to open editor: {}", e))?;

    if !status.success() {
        return Err("Editor exited with non-zero status".to_string());
    }

    Ok(())
}

/// Display the entire configuration
pub fn display_config(config_path: &PathBuf) -> Result<Config, String> {
    Config::load(config_path)
}

/// Handle the config init subcommand
pub fn handle_init() {
    let config_path = get_config_path();
    match init_config(&config_path) {
        Ok(_) => {
            println!("Initialized config file at: {}", config_path.display());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the config get subcommand
pub fn handle_get(key: Option<&str>, format: OutputFormat) {
    let config_path = get_config_path();

    match key {
        Some(k) => {
            // Get specific key
            match get_config_value(&config_path, k) {
                Ok(value) => println!("{}", value),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            // Display all config
            match display_config(&config_path) {
                Ok(config) => match crate::output::format_output(&config, format) {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        eprintln!("Error formatting output: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Handle the config set subcommand
pub fn handle_set(key: Option<&str>, value: Option<&str>) {
    let config_path = get_config_path();

    match (key, value) {
        (Some(k), Some(v)) => {
            // Set specific key
            match set_config_value(&config_path, k, v) {
                Ok(_) => println!("Set {} = {}", k, v),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        (None, None) => {
            // Open editor
            match edit_config(&config_path) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!(
                "Error: Invalid arguments. Use 'rex config set <key> <value>' or 'rex config set' to edit."
            );
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
