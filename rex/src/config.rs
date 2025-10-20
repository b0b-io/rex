use crate::output::{Formattable, OutputFormat};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tabled::Tabled;
use url::Url;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Style configuration
    #[serde(default)]
    pub style: StyleConfig,
    /// Registry configuration
    #[serde(default)]
    pub registries: RegistriesConfig,
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

/// Registries configuration section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistriesConfig {
    /// Default registry name
    pub default: Option<String>,
    /// List of configured registries
    #[serde(default)]
    pub list: Vec<RegistryEntry>,
}

/// A single registry entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Tabled)]
pub struct RegistryEntry {
    /// Registry name
    #[tabled(rename = "NAME")]
    pub name: String,
    /// Registry URL
    #[tabled(rename = "URL")]
    pub url: String,
}

/// Registry entry with default marker for display purposes
#[derive(Debug, Tabled, Serialize)]
pub struct RegistryDisplay {
    #[tabled(rename = "NAME")]
    pub name: String,
    #[tabled(rename = "URL")]
    pub url: String,
    #[tabled(rename = "DEFAULT")]
    pub default: String,
}

impl Formattable for RegistryDisplay {
    fn format_pretty(&self) -> String {
        let default_marker = if !self.default.is_empty() {
            " (default)"
        } else {
            ""
        };
        format!("Name: {}\nURL: {}{}", self.name, self.url, default_marker)
    }
}

impl Formattable for RegistryEntry {
    fn format_pretty(&self) -> String {
        format!("{:20} {}", self.name, self.url)
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

/// Validate and normalize a registry URL
///
/// This function validates that the URL is well-formed and uses http or https scheme.
/// If no scheme is provided, it defaults to http://.
///
/// # Arguments
///
/// * `url_str` - The URL string to validate
///
/// # Returns
///
/// Returns the normalized URL string on success, or an error message on failure.
///
/// # Examples
///
/// ```
/// # use std::path::PathBuf;
/// # use rex::config::validate_registry_url;
/// assert!(validate_registry_url("https://registry-1.docker.io").is_ok());
/// assert!(validate_registry_url("localhost:5000").is_ok());
/// assert!(validate_registry_url("http:://bad").is_err());
/// ```
pub fn validate_registry_url(url_str: &str) -> Result<String, String> {
    // Try to parse as-is first
    let url_to_parse = if url_str.contains("://") {
        url_str.to_string()
    } else {
        // Add default http:// scheme if no scheme provided
        format!("http://{}", url_str)
    };

    // Parse and validate the URL
    let parsed_url =
        Url::parse(&url_to_parse).map_err(|e| format!("Invalid URL '{}': {}", url_str, e))?;

    // Validate that the scheme is http or https
    match parsed_url.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(format!(
                "Invalid URL scheme '{}'. Only 'http' and 'https' are supported.",
                scheme
            ));
        }
    }

    // Validate that there's a host
    if parsed_url.host_str().is_none() {
        return Err(format!("Invalid URL '{}': missing host", url_str));
    }

    Ok(parsed_url.to_string())
}

/// Add a new registry to the configuration
pub fn add_registry(config_path: &PathBuf, name: &str, url: &str) -> Result<(), String> {
    // Load existing config or create default
    let mut config = if config_path.exists() {
        Config::load(config_path)?
    } else {
        Config::default()
    };

    // Check if registry with this name already exists
    if config.registries.list.iter().any(|r| r.name == name) {
        return Err(format!("Registry '{}' already exists", name));
    }

    // Validate and normalize URL
    let normalized_url = validate_registry_url(url)?;

    // Add the new registry
    config.registries.list.push(RegistryEntry {
        name: name.to_string(),
        url: normalized_url,
    });

    // Set as default if this is the first registry
    if config.registries.list.len() == 1 {
        config.registries.default = Some(name.to_string());
    }

    // Save config
    config.save(config_path)?;

    Ok(())
}

/// Remove a registry from the configuration
pub fn remove_registry(config_path: &PathBuf, name: &str) -> Result<(), String> {
    // Load existing config
    let mut config = Config::load(config_path)?;

    // Find the registry index
    let registry_index = config
        .registries
        .list
        .iter()
        .position(|r| r.name == name)
        .ok_or_else(|| format!("Registry '{}' not found", name))?;

    // Remove the registry
    config.registries.list.remove(registry_index);

    // Clear default if it was the removed registry
    if config.registries.default.as_ref() == Some(&name.to_string()) {
        config.registries.default = None;
    }

    // Save config
    config.save(config_path)?;

    Ok(())
}

/// Set the default registry
pub fn set_default_registry(config_path: &PathBuf, name: &str) -> Result<(), String> {
    // Load existing config
    let mut config = Config::load(config_path)?;

    // Check if registry exists
    if !config.registries.list.iter().any(|r| r.name == name) {
        return Err(format!("Registry '{}' not found", name));
    }

    // Set as default
    config.registries.default = Some(name.to_string());

    // Save config
    config.save(config_path)?;

    Ok(())
}

/// Show details of a specific registry
pub fn show_registry(config_path: &PathBuf, name: &str) -> Result<RegistryDisplay, String> {
    // Load existing config
    let config = Config::load(config_path)?;

    // Find the registry
    let registry = config
        .registries
        .list
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| format!("Registry '{}' not found", name))?;

    // Create display with default marker
    let is_default = config.registries.default.as_ref() == Some(&name.to_string());
    Ok(RegistryDisplay {
        name: registry.name.clone(),
        url: registry.url.clone(),
        default: if is_default {
            "*".to_string()
        } else {
            String::new()
        },
    })
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

/// List all registries
pub fn list_registries(config_path: &PathBuf) -> Result<Vec<RegistryDisplay>, String> {
    let config = Config::load(config_path)?;

    // Create display list with default markers
    let registries: Vec<RegistryDisplay> = config
        .registries
        .list
        .iter()
        .map(|r| {
            let is_default = config.registries.default.as_ref() == Some(&r.name);
            RegistryDisplay {
                name: r.name.clone(),
                url: r.url.clone(),
                default: if is_default {
                    "*".to_string()
                } else {
                    String::new()
                },
            }
        })
        .collect();

    Ok(registries)
}

/// Handle the registry add subcommand
pub fn handle_registry_add(name: &str, url: &str) {
    let config_path = get_config_path();
    match add_registry(&config_path, name, url) {
        Ok(_) => println!("Added registry '{}' at {}", name, url),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry remove subcommand
pub fn handle_registry_remove(name: &str) {
    let config_path = get_config_path();
    match remove_registry(&config_path, name) {
        Ok(_) => println!("Removed registry '{}'", name),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry set-default subcommand
pub fn handle_registry_set_default(name: &str) {
    let config_path = get_config_path();
    match set_default_registry(&config_path, name) {
        Ok(_) => println!("Set '{}' as default registry", name),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry show subcommand
pub fn handle_registry_show(name: &str, format: OutputFormat) {
    let config_path = get_config_path();
    match show_registry(&config_path, name) {
        Ok(registry) => match crate::output::format_output(&registry, format) {
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

/// Handle the registry list subcommand
pub fn handle_registry_list(format: OutputFormat) {
    let config_path = get_config_path();
    match list_registries(&config_path) {
        Ok(registries) => {
            if registries.is_empty() {
                println!("No registries configured.");
                return;
            }

            match format {
                OutputFormat::Pretty => {
                    use tabled::Table;
                    let table = Table::new(&registries).to_string();
                    println!("{}", table);
                }
                OutputFormat::Json => match serde_json::to_string_pretty(&registries) {
                    Ok(json) => println!("{}", json),
                    Err(e) => {
                        eprintln!("Error formatting JSON: {}", e);
                        std::process::exit(1);
                    }
                },
                OutputFormat::Yaml => match serde_yaml::to_string(&registries) {
                    Ok(yaml) => print!("{}", yaml),
                    Err(e) => {
                        eprintln!("Error formatting YAML: {}", e);
                        std::process::exit(1);
                    }
                },
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
