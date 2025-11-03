//! Application configuration.
//!
//! This module manages application configuration with sensible defaults,
//! loading from a YAML file and merging with environment variables.

use crate::error::{Result, RexError};
use config::{Config as ConfigRs, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(test)]
mod tests;

/// Root configuration structure.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
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

impl Config {
    /// Parses a `Config` from a YAML string.
    ///
    /// This function is primarily used for testing.
    pub fn from_yaml_str(s: &str) -> Result<Self> {
        let builder = ConfigRs::builder()
            // Add default values
            .add_source(ConfigRs::try_from(&Config::default())?)
            // Merge with YAML string
            .add_source(File::from_str(s, FileFormat::Yaml));

        Self::from_builder(builder)
    }

    /// Loads a `Config` from an optional file path.
    ///
    /// If the path is `None`, it will try to load from the default location.
    /// If the file does not exist, a default configuration is returned.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let mut builder = ConfigRs::builder()
            // Add default values
            .add_source(ConfigRs::try_from(&Config::default())?);

        // TODO: Add logic to load from default path if path is None
        // For now, we only load from the specified path if it exists.
        if let Some(p) = path {
            builder = builder.add_source(File::from(p).required(true));
        }

        Self::from_builder(builder)
    }

    /// Creates a `Config` from a `config::ConfigBuilder`.
    fn from_builder(builder: config::ConfigBuilder<config::builder::DefaultState>) -> Result<Self> {
        builder
            .build()
            .and_then(|cfg| cfg.try_deserialize())
            .map_err(|e| {
                RexError::config_with_source(
                    "Failed to deserialize configuration",
                    None::<String>,
                    e,
                )
            })
    }
}

/// Output formatting settings.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Output {
    #[serde(default)]
    pub format: OutputFormat,

    #[serde(default)]
    pub color: ColorChoice,
}

/// Enum for output formats.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Pretty,

    Json,

    Yaml,
}

/// Enum for color output choices.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColorChoice {
    #[default]
    Auto,

    Always,

    Never,
}

/// Network settings.

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]

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
    3600 // 1 hour - repositories don't change frequently
}

fn default_cache_ttl_tags() -> u64 {
    1800 // 30 minutes - tags can be pushed during development
}

fn default_cache_ttl_manifest() -> u64 {
    86400 // 1 day - reasonable for tag-based lookups
}

fn default_cache_ttl_config() -> u64 {
    31536000 // 365 days - effectively forever (immutable, content-addressed)
}

/// Cache size limits.

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Registries {
    #[serde(default)]
    pub current: Option<String>,

    #[serde(default)]
    pub list: Vec<Registry>,
}

/// Configuration for a single registry.

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]

pub struct Registry {
    pub name: String,

    pub url: String,

    #[serde(default)]
    pub insecure: bool,
}
