//! Application configuration.
//!
//! This module manages application configuration with sensible defaults,
//! loading from a YAML file.

use crate::error::{Result, RexError};
use serde::Deserialize;
use std::path::Path;

#[cfg(test)]
mod tests;

/// Root configuration structure.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub output: Output,
    #[serde(default)]
    pub network: Network,
    #[serde(default)]
    pub cache: Cache,
    #[serde(default)]
    pub tui: Tui,
    #[serde(default)]
    pub registries: Registries,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output: Output::default(),
            network: Network::default(),
            cache: Cache::default(),
            tui: Tui::default(),
            registries: Registries::default(),
        }
    }
}

impl Config {
    /// Parses a `Config` from a YAML string.
    pub fn from_str(s: &str) -> Result<Self> {
        serde_yaml::from_str(s).map_err(|e| {
            RexError::config_with_source("Failed to parse configuration", None::<String>, e)
        })
    }

    /// Loads a `Config` from a file path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            RexError::config_with_source(
                "Failed to read configuration file",
                Some(path.as_ref().display().to_string()),
                e,
            )
        })?;
        Self::from_str(&contents)
    }
}

/// Output formatting settings.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Output {
    #[serde(default)]
    pub format: OutputFormat,
    #[serde(default)]
    pub color: ColorChoice,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            format: OutputFormat::default(),
            color: ColorChoice::default(),
        }
    }
}

/// Enum for output formats.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Pretty,
    Json,
    Yaml,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Pretty
    }
}

/// Enum for color output choices.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl Default for ColorChoice {
    fn default() -> Self {
        Self::Auto
    }
}

/// Network settings.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Network {
    #[serde(default = "default_network_timeout")]
    pub timeout: u64,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            timeout: default_network_timeout(),
        }
    }
}

fn default_network_timeout() -> u64 {
    30
}

/// Cache settings.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Cache {
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub ttl: CacheTtl,
    #[serde(default)]
    pub limits: CacheLimits,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            ttl: CacheTtl::default(),
            limits: CacheLimits::default(),
        }
    }
}

fn default_cache_enabled() -> bool {
    true
}

/// Cache time-to-live (TTL) settings in seconds.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CacheTtl {
    #[serde(default = "default_cache_ttl_catalog")]
    pub catalog: u64,
    #[serde(default = "default_cache_ttl_tags")]
    pub tags: u64,
    #[serde(default = "default_cache_ttl_manifest")]
    pub manifest: u64,
    #[serde(default = "default_cache_ttl_config")]
    pub config: u64,
}

impl Default for CacheTtl {
    fn default() -> Self {
        Self {
            catalog: default_cache_ttl_catalog(),
            tags: default_cache_ttl_tags(),
            manifest: default_cache_ttl_manifest(),
            config: default_cache_ttl_config(),
        }
    }
}

fn default_cache_ttl_catalog() -> u64 {
    300
}

fn default_cache_ttl_tags() -> u64 {
    300
}

fn default_cache_ttl_manifest() -> u64 {
    86400
}

fn default_cache_ttl_config() -> u64 {
    86400
}

/// Cache size limits.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CacheLimits {
    #[serde(default = "default_cache_limits_memory_entries")]
    pub memory_entries: usize,
    #[serde(default = "default_cache_limits_disk_entries")]
    pub disk_entries: usize,
}

impl Default for CacheLimits {
    fn default() -> Self {
        Self {
            memory_entries: default_cache_limits_memory_entries(),
            disk_entries: default_cache_limits_disk_entries(),
        }
    }
}

fn default_cache_limits_memory_entries() -> usize {
    1000
}

fn default_cache_limits_disk_entries() -> usize {
    10000
}

/// Terminal UI settings.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Tui {
    #[serde(default = "default_tui_theme")]
    pub theme: String,
    #[serde(default = "default_tui_vim_bindings")]
    pub vim_bindings: bool,
}

impl Default for Tui {
    fn default() -> Self {
        Self {
            theme: default_tui_theme(),
            vim_bindings: default_tui_vim_bindings(),
        }
    }
}

fn default_tui_theme() -> String {
    "dark".to_string()
}

fn default_tui_vim_bindings() -> bool {
    true
}

/// Registry management settings.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Registries {
    #[serde(default)]
    pub current: Option<String>,
    #[serde(default)]
    pub list: Vec<Registry>,
}

impl Default for Registries {
    fn default() -> Self {
        Self {
            current: None,
            list: Vec::new(),
        }
    }
}

/// Configuration for a single registry.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Registry {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub insecure: bool,
}
