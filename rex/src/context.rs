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

/// Verbosity level for output
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerbosityLevel {
    /// Normal output (no -v flag)
    Normal = 0,
    /// Basic verbose output (-v)
    /// Shows what operations are happening
    Verbose = 1,
    /// Detailed verbose output (-vv)
    /// Shows HTTP requests/responses
    VeryVerbose = 2,
    /// Maximum verbosity (-vvv)
    /// Shows all details, timing, traces
    Trace = 3,
}

impl VerbosityLevel {
    /// Create from count of -v flags
    pub fn from_count(count: u8) -> Self {
        match count {
            0 => VerbosityLevel::Normal,
            1 => VerbosityLevel::Verbose,
            2 => VerbosityLevel::VeryVerbose,
            _ => VerbosityLevel::Trace,
        }
    }
}

/// Application context with resolved configuration and runtime state
#[derive(Debug, Clone)]
pub struct AppContext {
    /// Resolved configuration
    pub config: Config,
    /// Verbosity level
    pub verbosity: VerbosityLevel,
}

impl AppContext {
    /// Build context with precedence: defaults > config file > env vars > CLI flags
    pub fn build(cli_color: ColorChoice, verbosity: VerbosityLevel) -> Self {
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
        if let Ok(concurrency) = env::var("REX_CONCURRENCY")
            && let Ok(value) = concurrency.parse::<usize>()
        {
            config.concurrency = value;
        }
        // TUI environment variables
        if let Ok(theme) = env::var("REX_TUI_THEME") {
            config.tui.theme = theme;
        }
        if let Ok(vim_mode) = env::var("REX_TUI_VIM_MODE") {
            config.tui.vim_mode = vim_mode.parse().unwrap_or(config.tui.vim_mode);
        }
        if let Ok(max_workers) = env::var("REX_TUI_MAX_WORKERS")
            && let Ok(value) = max_workers.parse::<usize>()
        {
            config.tui.max_workers = value;
        }
        if let Ok(poll_interval) = env::var("REX_TUI_POLL_INTERVAL")
            && let Ok(value) = poll_interval.parse::<u64>()
        {
            config.tui.poll_interval = value;
        }

        // 4. Apply CLI flag overrides (highest priority)
        // Only override if not Auto (which is the default from clap)
        // This way we respect config file when user doesn't explicitly set --color
        if cli_color != ColorChoice::Auto || env::var("REX_COLOR").is_ok() {
            config.style.color = cli_color;
        }

        Self { config, verbosity }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_level_from_count() {
        assert_eq!(VerbosityLevel::from_count(0), VerbosityLevel::Normal);
        assert_eq!(VerbosityLevel::from_count(1), VerbosityLevel::Verbose);
        assert_eq!(VerbosityLevel::from_count(2), VerbosityLevel::VeryVerbose);
        assert_eq!(VerbosityLevel::from_count(3), VerbosityLevel::Trace);
        assert_eq!(VerbosityLevel::from_count(4), VerbosityLevel::Trace); // Max at 3
        assert_eq!(VerbosityLevel::from_count(100), VerbosityLevel::Trace);
    }

    #[test]
    fn test_verbosity_level_ordering() {
        assert!(VerbosityLevel::Normal < VerbosityLevel::Verbose);
        assert!(VerbosityLevel::Verbose < VerbosityLevel::VeryVerbose);
        assert!(VerbosityLevel::VeryVerbose < VerbosityLevel::Trace);

        assert!(VerbosityLevel::Trace >= VerbosityLevel::Normal);
        assert!(VerbosityLevel::Trace >= VerbosityLevel::Verbose);
        assert!(VerbosityLevel::Trace >= VerbosityLevel::VeryVerbose);
        assert!(VerbosityLevel::Trace >= VerbosityLevel::Trace);
    }

    #[test]
    fn test_verbosity_level_equality() {
        assert_eq!(VerbosityLevel::Normal, VerbosityLevel::Normal);
        assert_eq!(VerbosityLevel::Verbose, VerbosityLevel::Verbose);
        assert_ne!(VerbosityLevel::Normal, VerbosityLevel::Verbose);
    }

    #[test]
    fn test_app_context_build_with_normal_verbosity() {
        let ctx = AppContext::build(ColorChoice::Auto, VerbosityLevel::Normal);
        assert_eq!(ctx.verbosity, VerbosityLevel::Normal);
    }

    #[test]
    fn test_app_context_build_with_verbose() {
        let ctx = AppContext::build(ColorChoice::Auto, VerbosityLevel::Verbose);
        assert_eq!(ctx.verbosity, VerbosityLevel::Verbose);
    }

    #[test]
    fn test_app_context_build_with_very_verbose() {
        let ctx = AppContext::build(ColorChoice::Auto, VerbosityLevel::VeryVerbose);
        assert_eq!(ctx.verbosity, VerbosityLevel::VeryVerbose);
    }

    #[test]
    fn test_app_context_build_with_trace() {
        let ctx = AppContext::build(ColorChoice::Auto, VerbosityLevel::Trace);
        assert_eq!(ctx.verbosity, VerbosityLevel::Trace);
    }

    #[test]
    fn test_app_context_verbosity_is_preserved() {
        let ctx = AppContext::build(ColorChoice::Never, VerbosityLevel::VeryVerbose);
        assert_eq!(ctx.verbosity, VerbosityLevel::VeryVerbose);
        assert_eq!(ctx.config.style.color, ColorChoice::Never);
    }
}
