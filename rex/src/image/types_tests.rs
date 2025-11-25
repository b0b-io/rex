//! Tests for shared image types.

use super::*;
use chrono::{TimeZone, Utc};

#[test]
fn test_tag_info_new_formats_digest_correctly() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abcdef1234567890abcdef".to_string(),
        1024,
        None,
        vec![],
    );

    // Should truncate to 12 chars after sha256:
    assert_eq!(tag.digest, "abcdef123456");
}

#[test]
fn test_tag_info_new_handles_short_digest() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc".to_string(),
        1024,
        None,
        vec![],
    );

    // Should handle short digests gracefully (takes first 12 chars of whole string)
    assert_eq!(tag.digest, "sha256:abc");
}

#[test]
fn test_tag_info_new_handles_placeholder_digest() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:...".to_string(),
        1024,
        None,
        vec![],
    );

    assert_eq!(tag.digest, "...");
}

#[test]
fn test_tag_info_new_handles_na_digest() {
    let tag = TagInfo::new("latest".to_string(), "N/A".to_string(), 1024, None, vec![]);

    assert_eq!(tag.digest, "N/A");
}

#[test]
fn test_tag_info_new_formats_size_correctly() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        7340032, // ~7 MB
        None,
        vec![],
    );

    // Should be formatted as human-readable size
    assert!(tag.size.contains("MB") || tag.size.contains("MiB"));
}

#[test]
fn test_tag_info_new_formats_created_with_timestamp() {
    let created = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        Some(created),
        vec![],
    );

    // Should be formatted as relative time (not "N/A")
    assert_ne!(tag.created, "N/A");
}

#[test]
fn test_tag_info_new_formats_created_without_timestamp() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        None,
        vec![],
    );

    assert_eq!(tag.created, "N/A");
}

#[test]
fn test_tag_info_new_formats_single_platform() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        None,
        vec!["linux/amd64".to_string()],
    );

    assert_eq!(tag.platforms, "linux/amd64");
}

#[test]
fn test_tag_info_new_formats_two_platforms() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        None,
        vec!["linux/amd64".to_string(), "linux/arm64".to_string()],
    );

    assert_eq!(tag.platforms, "linux/amd64, linux/arm64");
}

#[test]
fn test_tag_info_new_formats_many_platforms() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        None,
        vec![
            "linux/amd64".to_string(),
            "linux/arm64".to_string(),
            "linux/arm/v7".to_string(),
        ],
    );

    assert_eq!(tag.platforms, "3 platforms");
}

#[test]
fn test_tag_info_new_formats_empty_platforms() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        None,
        vec![],
    );

    assert_eq!(tag.platforms, "N/A");
}

#[test]
fn test_tag_info_new_preserves_raw_timestamp() {
    let created = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        Some(created),
        vec![],
    );

    assert_eq!(tag.created_timestamp, Some(created));
}

#[test]
fn test_tag_info_new_handles_none_timestamp() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        None,
        vec![],
    );

    assert_eq!(tag.created_timestamp, None);
}
