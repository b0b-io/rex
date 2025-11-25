//! Shared types for image metadata.

use serde::Serialize;
use tabled::Tabled;

/// Repository (image) information for listing.
///
/// This type is shared between CLI and TUI to ensure consistent data representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tabled)]
pub struct RepositoryItem {
    /// Repository name
    #[tabled(rename = "NAME")]
    pub name: String,

    /// Number of tags in this repository
    #[tabled(rename = "TAGS")]
    pub tag_count: usize,

    /// Total size of the most recent tag (in bytes, formatted for display)
    #[tabled(rename = "SIZE")]
    pub total_size_display: String,

    /// Last updated timestamp (formatted for display)
    #[tabled(rename = "UPDATED")]
    pub last_updated: String,

    /// Raw size for sorting (not displayed)
    #[tabled(skip)]
    #[serde(skip)]
    pub total_size: u64,

    /// Raw timestamp for sorting (not displayed)
    #[tabled(skip)]
    #[serde(skip)]
    pub last_updated_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl RepositoryItem {
    /// Create a new RepositoryItem with formatted display values.
    ///
    /// # Arguments
    ///
    /// * `name` - Repository name
    /// * `tag_count` - Number of tags
    /// * `total_size` - Total size in bytes (will be formatted)
    /// * `last_updated` - Optional last updated timestamp (will be formatted)
    pub fn new(
        name: String,
        tag_count: usize,
        total_size: u64,
        last_updated: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        // Format size
        let total_size_display = if total_size == 0 {
            "N/A".to_string()
        } else {
            librex::format::format_size(total_size)
        };

        // Format last updated time
        let last_updated_display = last_updated
            .map(|ts| librex::format::format_timestamp(&ts))
            .unwrap_or_else(|| "N/A".to_string());

        Self {
            name,
            tag_count,
            total_size_display,
            last_updated: last_updated_display,
            total_size,
            last_updated_timestamp: last_updated,
        }
    }
}

/// Tag information for a specific image.
///
/// This type is shared between CLI and TUI to ensure consistent formatting
/// and data representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tabled)]
pub struct TagInfo {
    /// Tag name
    #[tabled(rename = "TAG")]
    pub tag: String,

    /// Manifest digest (truncated for display)
    #[tabled(rename = "DIGEST")]
    pub digest: String,

    /// Total size (formatted for display)
    #[tabled(rename = "SIZE")]
    pub size: String,

    /// Created timestamp (relative format)
    #[tabled(rename = "CREATED")]
    pub created: String,

    /// Platform(s) available
    #[tabled(rename = "PLATFORM")]
    pub platforms: String,

    /// Raw created timestamp for sorting (not displayed in table)
    #[tabled(skip)]
    #[serde(skip)]
    pub created_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl TagInfo {
    /// Create a new TagInfo with formatted display values.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag name
    /// * `digest` - The full manifest digest (will be truncated to 12 chars)
    /// * `size` - Total size in bytes (will be formatted as human-readable)
    /// * `created` - Optional created timestamp (will be formatted as relative time)
    /// * `platforms` - List of platforms (will be formatted as comma-separated or count)
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::image::TagInfo;
    /// use chrono::Utc;
    ///
    /// let tag = TagInfo::new(
    ///     "latest".to_string(),
    ///     "sha256:abcdef1234567890".to_string(),
    ///     7340032,
    ///     Some(Utc::now()),
    ///     vec!["linux/amd64".to_string()],
    /// );
    ///
    /// assert_eq!(tag.tag, "latest");
    /// assert_eq!(tag.digest, "abcdef123456"); // Truncated to 12 chars
    /// ```
    pub fn new(
        tag: String,
        digest: String,
        size: u64,
        created: Option<chrono::DateTime<chrono::Utc>>,
        platforms: Vec<String>,
    ) -> Self {
        // Extract short digest (12 hex chars after sha256:) for compact display
        let digest_display = if digest == "sha256:..." {
            // Placeholder digest when actual digest is not available
            "...".to_string()
        } else if digest.starts_with("sha256:") && digest.len() >= 19 {
            // Extract 12 chars after "sha256:" prefix (7 + 12 = 19)
            digest[7..19].to_string()
        } else if digest == "N/A" {
            "N/A".to_string()
        } else {
            // Fallback for other formats
            digest.chars().take(12).collect()
        };

        // Format size using librex format module
        let size_display = librex::format::format_size(size);

        // Format created time using librex format module
        let created_display = created
            .map(|c| librex::format::format_timestamp(&c))
            .unwrap_or_else(|| "N/A".to_string());

        // Format platforms (comma-separated or show count if > 2)
        let platforms_display = if platforms.is_empty() {
            "N/A".to_string()
        } else if platforms.len() <= 2 {
            platforms.join(", ")
        } else {
            format!("{} platforms", platforms.len())
        };

        Self {
            tag,
            digest: digest_display,
            size: size_display,
            created: created_display,
            platforms: platforms_display,
            created_timestamp: created,
        }
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
