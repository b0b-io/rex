use crate::format::{ColorChoice, OutputFormat};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Style configuration
    #[serde(default)]
    pub style: StyleConfig,
    /// TUI configuration
    #[serde(default)]
    pub tui: TuiConfig,
    /// Registry configuration
    #[serde(default)]
    pub registries: RegistriesConfig,
    /// Cache directory path
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
    /// Maximum number of concurrent requests for parallel operations
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            style: StyleConfig::default(),
            tui: TuiConfig::default(),
            registries: RegistriesConfig::default(),
            cache_dir: default_cache_dir(),
            concurrency: default_concurrency(),
        }
    }
}

fn default_cache_dir() -> String {
    get_default_cache_dir().to_string_lossy().to_string()
}

fn default_concurrency() -> usize {
    8
}

/// TUI configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Theme: "dark" or "light"
    #[serde(default = "default_tui_theme")]
    pub theme: String,
    /// Enable vim-style navigation (hjkl)
    #[serde(default = "default_tui_vim_mode")]
    pub vim_mode: bool,
    /// Maximum concurrent worker threads
    #[serde(default = "default_tui_max_workers")]
    pub max_workers: usize,
    /// Event polling interval in milliseconds
    #[serde(default = "default_tui_poll_interval")]
    pub poll_interval: u64,
}

fn default_tui_theme() -> String {
    "dark".to_string()
}

fn default_tui_vim_mode() -> bool {
    false
}

fn default_tui_max_workers() -> usize {
    10
}

fn default_tui_poll_interval() -> u64 {
    100
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            theme: default_tui_theme(),
            vim_mode: default_tui_vim_mode(),
            max_workers: default_tui_max_workers(),
            poll_interval: default_tui_poll_interval(),
        }
    }
}

/// Style configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    /// Output format: pretty, json, yaml
    #[serde(default = "default_format")]
    pub format: OutputFormat,
    /// Color output control: auto, always, never
    #[serde(default = "default_color")]
    pub color: ColorChoice,
}

fn default_format() -> OutputFormat {
    OutputFormat::Pretty
}

fn default_color() -> ColorChoice {
    ColorChoice::Auto
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Pretty,
            color: ColorChoice::Auto,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistryEntry {
    /// Registry name
    pub name: String,
    /// Registry URL
    pub url: String,
    /// Enable Docker Hub compatibility mode (adds "library/" prefix for simple names)
    /// Default: false (works with Zot, GHCR, and most registries)
    #[serde(default)]
    pub dockerhub_compat: bool,
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
        }),
        ["style", "color"] => Ok(config.style.color.to_string()),
        ["tui", "theme"] => Ok(config.tui.theme.clone()),
        ["tui", "vim_mode"] => Ok(config.tui.vim_mode.to_string()),
        ["tui", "max_workers"] => Ok(config.tui.max_workers.to_string()),
        ["tui", "poll_interval"] => Ok(config.tui.poll_interval.to_string()),
        ["cache_dir"] => Ok(config.cache_dir.clone()),
        ["concurrency"] => Ok(config.concurrency.to_string()),
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
            config.style.color = ColorChoice::from(value);
        }
        ["tui", "theme"] => {
            config.tui.theme = value.to_string();
        }
        ["tui", "vim_mode"] => {
            config.tui.vim_mode = value.parse::<bool>().map_err(|_| {
                format!("Invalid vim_mode value '{}': must be true or false", value)
            })?;
        }
        ["tui", "max_workers"] => {
            config.tui.max_workers = value.parse::<usize>().map_err(|_| {
                format!(
                    "Invalid max_workers value '{}': must be a positive integer",
                    value
                )
            })?;
        }
        ["tui", "poll_interval"] => {
            config.tui.poll_interval = value.parse::<u64>().map_err(|_| {
                format!(
                    "Invalid poll_interval value '{}': must be a positive integer",
                    value
                )
            })?;
        }
        ["cache_dir"] => {
            config.cache_dir = value.to_string();
        }
        ["concurrency"] => {
            config.concurrency = value.parse::<usize>().map_err(|_| {
                format!(
                    "Invalid concurrency value '{}': must be a positive integer",
                    value
                )
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

/// Get credentials path
pub fn get_credentials_path() -> PathBuf {
    if let Ok(creds_path) = env::var("REX_CREDENTIALS") {
        return PathBuf::from(creds_path);
    }

    // Default to ~/.config/rex/credentials.toml
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("rex").join("credentials.toml")
    } else {
        // Fallback to current directory
        PathBuf::from("credentials.toml")
    }
}

/// Get the default cache directory
///
/// Returns the default cache directory for rex.
/// Uses platform-specific cache directory (~/.cache/rex on Linux, ~/Library/Caches/rex on macOS, etc.)
pub fn get_default_cache_dir() -> PathBuf {
    if let Some(cache_dir) = dirs::cache_dir() {
        cache_dir.join("rex")
    } else {
        // Fallback to temp directory
        env::temp_dir().join("rex-cache")
    }
}

/// Get the cache directory for a specific registry
pub fn get_registry_cache_dir(registry_url: &str) -> Result<PathBuf, String> {
    let config_path = get_config_path();

    // Load config to get cache_dir
    let cache_base = if let Ok(cfg) = Config::load(&config_path) {
        PathBuf::from(cfg.cache_dir)
    } else {
        // Use default if config doesn't exist
        get_default_cache_dir()
    };

    // Create a safe directory name from the registry URL
    // Replace special characters with underscores
    let safe_name = registry_url
        .replace("://", "_")
        .replace(['/', ':', '.'], "_");

    Ok(cache_base.join(safe_name))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
