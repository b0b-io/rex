use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::io::IsTerminal;

/// Trait for output formatting that can be TTY-aware or plain text
pub trait OutputFormatter: Send + Sync {
    /// Print a success message
    fn success(&self, message: &str);

    /// Print an error message
    fn error(&self, message: &str);

    /// Print a warning message
    fn warning(&self, message: &str);

    /// Create a spinner for indeterminate progress
    fn spinner(&self, message: &str) -> ProgressBar;

    /// Create a progress bar for determinate progress
    fn progress_bar(&self, len: u64, message: &str) -> ProgressBar;

    /// Finish a progress operation with a message
    fn finish_progress(&self, pb: ProgressBar, message: &str);
}

/// TTY-aware formatter with colors and progress indicators
pub struct TtyFormatter;

impl OutputFormatter for TtyFormatter {
    fn success(&self, message: &str) {
        println!("{} {}", "✓".green().bold(), message);
    }

    fn error(&self, message: &str) {
        eprintln!("{} {}", "✗".red().bold(), message);
    }

    fn warning(&self, message: &str) {
        println!("{} {}", "⚠".yellow().bold(), message);
    }

    fn spinner(&self, message: &str) -> ProgressBar {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        spinner
    }

    fn progress_bar(&self, len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("█▓▒░ "),
        );
        pb.set_message(message.to_string());
        pb
    }

    fn finish_progress(&self, pb: ProgressBar, message: &str) {
        pb.finish_with_message(format!("{} {}", "✓".green(), message));
    }
}

/// Plain text formatter for non-TTY output (piped, scripted)
pub struct PlainFormatter;

impl OutputFormatter for PlainFormatter {
    fn success(&self, message: &str) {
        println!("✓ {}", message);
    }

    fn error(&self, message: &str) {
        eprintln!("✗ {}", message);
    }

    fn warning(&self, message: &str) {
        println!("⚠ {}", message);
    }

    fn spinner(&self, message: &str) -> ProgressBar {
        println!("{}", message);
        ProgressBar::hidden()
    }

    fn progress_bar(&self, len: u64, message: &str) -> ProgressBar {
        println!("{} (0/{})", message, len);
        ProgressBar::hidden()
    }

    fn finish_progress(&self, pb: ProgressBar, message: &str) {
        pb.finish();
        println!("✓ {}", message);
    }
}

/// Create the appropriate formatter based on TTY and environment
pub fn create_formatter() -> Box<dyn OutputFormatter> {
    // Check if NO_COLOR is set
    if std::env::var("NO_COLOR").is_ok() {
        return Box::new(PlainFormatter);
    }

    // Check if stdout OR stderr is a terminal (since we output to both)
    if std::io::stdout().is_terminal() || std::io::stderr().is_terminal() {
        Box::new(TtyFormatter)
    } else {
        Box::new(PlainFormatter)
    }
}

// Legacy helper functions that use a lazily created formatter
// These are kept for backward compatibility

use std::sync::OnceLock;

static FORMATTER: OnceLock<Box<dyn OutputFormatter>> = OnceLock::new();

fn get_formatter() -> &'static dyn OutputFormatter {
    FORMATTER.get_or_init(|| create_formatter()).as_ref()
}

/// Check if we should use colors in output
pub fn should_color() -> bool {
    std::io::stdout().is_terminal() && std::env::var("NO_COLOR").is_err()
}

/// Print a success message with optional coloring
pub fn success(message: &str) {
    get_formatter().success(message);
}

/// Print an error message with optional coloring
pub fn error(message: &str) {
    get_formatter().error(message);
}

/// Print a warning message with optional coloring
#[allow(dead_code)]
pub fn warning(message: &str) {
    get_formatter().warning(message);
}

/// Colorize a checkmark for success if colors are enabled
pub fn checkmark() -> String {
    // Check should_color() directly to handle NO_COLOR changes in tests
    if should_color() {
        format!("{}", "✓".green())
    } else {
        "✓".to_string()
    }
}

/// Colorize an X mark for errors if colors are enabled
#[allow(dead_code)]
pub fn error_mark() -> String {
    if should_color() {
        format!("{}", "✗".red())
    } else {
        "✗".to_string()
    }
}

/// Output format for CLI commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human-readable pretty format
    Pretty,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

impl From<&str> for OutputFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "yaml" | "yml" => OutputFormat::Yaml,
            _ => OutputFormat::Pretty,
        }
    }
}

/// Trait for types that can be formatted for CLI output
pub trait Formattable: Serialize {
    /// Format the type for pretty (human-readable) output
    fn format_pretty(&self) -> String;
}

/// Format a single item for output
pub fn format_output<T: Formattable>(item: &T, format: OutputFormat) -> Result<String, String> {
    match format {
        OutputFormat::Pretty => Ok(item.format_pretty()),
        OutputFormat::Json => serde_json::to_string_pretty(item)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e)),
        OutputFormat::Yaml => {
            serde_yaml::to_string(item).map_err(|e| format!("Failed to serialize to YAML: {}", e))
        }
    }
}

/// Format a vector of items for output
#[allow(dead_code)]
pub fn format_output_vec<T: Formattable>(
    items: &[T],
    format: OutputFormat,
) -> Result<String, String> {
    match format {
        OutputFormat::Pretty => {
            let output: Vec<String> = items.iter().map(|item| item.format_pretty()).collect();
            Ok(output.join("\n"))
        }
        OutputFormat::Json => serde_json::to_string_pretty(items)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e)),
        OutputFormat::Yaml => {
            serde_yaml::to_string(items).map_err(|e| format!("Failed to serialize to YAML: {}", e))
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
