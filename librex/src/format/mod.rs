//! Human-readable data formatting utilities.
//!
//! This module provides functions for formatting data types like byte sizes
//! and timestamps into user-friendly, human-readable strings.

use chrono::{DateTime, Utc};
use chrono_humanize::Humanize;
use humansize::{BINARY, DECIMAL, format_size as format_size_human};

#[cfg(test)]
mod tests;

/// Formats a byte size into a human-readable string using binary units (KiB, MiB).
///
/// # Examples
///
/// ```
/// use librex::format::format_size;
///
/// let size = 1024 * 1024 * 5; // 5 MiB
/// assert_eq!(format_size(size), "5 MiB");
///
/// let size = 1024; // 1 KiB
/// assert_eq!(format_size(size), "1 KiB");
/// ```
pub fn format_size(size_bytes: u64) -> String {
    format_size_human(size_bytes, BINARY)
}

/// Formats a byte size into a human-readable string using decimal units (kB, MB).
///
/// # Examples
///
/// ```
/// use librex::format::format_size_decimal;
///
/// let size = 1000 * 1000 * 5; // 5 MB
/// assert_eq!(format_size_decimal(size), "5 MB");
///
/// let size = 1000; // 1 kB
/// assert_eq!(format_size_decimal(size), "1 kB");
/// ```
pub fn format_size_decimal(size_bytes: u64) -> String {
    format_size_human(size_bytes, DECIMAL)
}

/// Formats a timestamp into a human-readable relative string.
///
/// # Examples
///
/// ```
/// use librex::format::format_timestamp;
/// use chrono::{DateTime, Utc, Duration};
///
/// let now = Utc::now();
/// let one_day_ago = now - Duration::days(1);
/// let formatted = format_timestamp(&one_day_ago);
/// assert_eq!(formatted, "a day ago");
/// ```
pub fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.humanize()
}
