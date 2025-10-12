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
pub fn format_size(size_bytes: u64) -> String {
    format_size_human(size_bytes, BINARY)
}

/// Formats a byte size into a human-readable string using decimal units (kB, MB).
pub fn format_size_decimal(size_bytes: u64) -> String {
    format_size_human(size_bytes, DECIMAL)
}

/// Formats a timestamp into a human-readable relative string.
pub fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.humanize()
}
