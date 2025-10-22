use serde::{Deserialize, Serialize};

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
