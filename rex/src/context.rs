//! Application context that holds resolved configuration
//!
//! The context is built following the precedence order:
//! 1. Default values
//! 2. Config file values
//! 3. Environment variables
//! 4. CLI flags
//!
//! Once built, the context is passed as read-only throughout the application.

use crate::config::{self, Config};
use crate::format::ColorChoice;
use std::env;

/// Application context with resolved configuration and runtime state
#[derive(Debug, Clone)]
pub struct AppContext {
    /// Resolved configuration
    pub config: Config,
}

impl AppContext {
    /// Build context with precedence: defaults > config file > env vars > CLI flags
    pub fn build(cli_color: ColorChoice) -> Self {
        // 1. Start with defaults
        let mut config = Config::default();

        // 2. Load and merge config file if it exists
        let config_path = config::get_config_path();
        if let Ok(file_config) = Config::load(&config_path) {
            config = file_config;
        }

        // 3. Apply environment variable overrides
        if let Ok(color) = env::var("REX_COLOR") {
            config.style.color = ColorChoice::from(color.as_str());
        }
        if let Ok(cache_dir) = env::var("REX_CACHE_DIR") {
            config.cache_dir = cache_dir;
        }

        // 4. Apply CLI flag overrides (highest priority)
        // Only override if not Auto (which is the default from clap)
        // This way we respect config file when user doesn't explicitly set --color
        if cli_color != ColorChoice::Auto || env::var("REX_COLOR").is_ok() {
            config.style.color = cli_color;
        }

        Self { config }
    }
}
