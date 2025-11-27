use super::*;

#[test]
fn test_repository_item_creation() {
    let item = RepositoryItem::new("alpine".to_string(), 5, 1024, None);

    assert_eq!(item.name, "alpine");
    assert_eq!(item.tag_count, 5);
}

#[test]
fn test_repository_item_format_pretty() {
    let item = RepositoryItem::new("nginx".to_string(), 12, 2048, None);

    let formatted = item.format_pretty();
    assert!(formatted.contains("nginx"));
    assert!(formatted.contains("12"));
}

#[test]
fn test_repository_item_serialization() {
    let item = RepositoryItem::new("redis".to_string(), 3, 3072, None);

    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"name\":\"redis\""));
    assert!(json.contains("\"tag_count\":3"));
}

#[test]
fn test_get_registry_url_default() {
    // When no config exists, should return localhost:5000
    let result = get_registry_url();
    assert!(result.is_ok());
    // Result should be either a configured registry or localhost:5000
    let url = result.unwrap();
    assert!(!url.is_empty());
}

#[test]
fn test_tag_info_creation() {
    use chrono::{DateTime, Utc};
    let created = DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123def456".to_string(),
        1024000,
        Some(created),
        vec!["linux/amd64".to_string()],
    );

    assert_eq!(tag.tag, "latest");
    // Digest should be 12 chars (short format)
    assert_eq!(tag.digest, "abc123def456");
    assert!(tag.size.contains("MiB") || tag.size.contains("KiB"));
    assert!(!tag.created.is_empty());
    assert!(tag.platforms.contains("linux/amd64"));
}

#[test]
fn test_tag_info_format_pretty() {
    use chrono::{DateTime, Utc};
    let created = DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let tag = TagInfo::new(
        "v1.2.3".to_string(),
        "sha256:xyz789".to_string(),
        2048000,
        Some(created),
        vec!["linux/amd64".to_string(), "linux/arm64".to_string()],
    );

    let formatted = tag.format_pretty();
    assert!(formatted.contains("v1.2.3"));
    // Short digest gets first 12 chars of whole string (fallback behavior)
    assert!(formatted.contains("sha256:xyz78"));
}

#[test]
fn test_tag_info_serialization() {
    use chrono::{DateTime, Utc};
    let created = DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let tag = TagInfo::new(
        "3.19".to_string(),
        "sha256:multi123".to_string(),
        5120000,
        Some(created),
        vec!["linux/amd64".to_string()],
    );

    let json = serde_json::to_string(&tag).unwrap();
    assert!(json.contains("\"tag\":\"3.19\""));
    assert!(json.contains("digest"));
    assert!(json.contains("size"));
    assert!(json.contains("created"));
    assert!(json.contains("platforms"));
}

#[test]
fn test_tag_info_with_many_platforms() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:multi123".to_string(),
        1024000,
        None,
        vec![
            "linux/amd64".to_string(),
            "linux/arm64".to_string(),
            "linux/arm/v7".to_string(),
        ],
    );

    // Should show count for more than 2 platforms
    assert!(tag.platforms.contains("3 platforms"));
}

#[test]
fn test_tag_info_without_created() {
    let tag = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024000,
        None,
        vec!["linux/amd64".to_string()],
    );

    assert_eq!(tag.created, "N/A");
}

#[test]
fn test_tag_info_digest_shortening() {
    // Test with full sha256 digest
    let tag1 = TagInfo::new(
        "latest".to_string(),
        "sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b".to_string(),
        1024000,
        None,
        vec!["linux/amd64".to_string()],
    );
    assert_eq!(tag1.digest, "c5b1261d6d3e");

    // Test with N/A digest
    let tag2 = TagInfo::new(
        "latest".to_string(),
        "N/A".to_string(),
        1024000,
        None,
        vec!["linux/amd64".to_string()],
    );
    assert_eq!(tag2.digest, "N/A");

    // Test with short digest (less than 12 hex chars)
    // Falls through to fallback: take first 12 chars of entire string
    let tag3 = TagInfo::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024000,
        None,
        vec!["linux/amd64".to_string()],
    );
    assert_eq!(tag3.digest, "sha256:abc12");

    // Test with placeholder digest
    let tag4 = TagInfo::new(
        "latest".to_string(),
        "sha256:...".to_string(),
        1024000,
        None,
        vec!["linux/amd64".to_string()],
    );
    assert_eq!(tag4.digest, "...");
}

#[test]
fn test_image_details_creation() {
    let details = ImageDetails::new(
        "alpine:latest".to_string(),
        "sha256:abc123".to_string(),
        "OCI Image Manifest".to_string(),
        1024000,
        vec!["linux/amd64".to_string()],
        5,
        Some("2025-01-15T10:30:00Z".to_string()),
    );

    assert_eq!(details.reference, "alpine:latest");
    assert_eq!(details.digest, "sha256:abc123");
    assert_eq!(details.manifest_type, "OCI Image Manifest");
    assert_eq!(details.size, 1024000);
    assert_eq!(details.platforms, vec!["linux/amd64"]);
    assert_eq!(details.layers, 5);
    assert_eq!(details.created, Some("2025-01-15T10:30:00Z".to_string()));
}

#[test]
fn test_image_details_multi_platform() {
    let details = ImageDetails::new(
        "nginx:latest".to_string(),
        "sha256:def456".to_string(),
        "OCI Image Index (multi-platform)".to_string(),
        5120000,
        vec!["linux/amd64".to_string(), "linux/arm64".to_string()],
        2,
        None,
    );

    assert_eq!(details.reference, "nginx:latest");
    assert_eq!(details.manifest_type, "OCI Image Index (multi-platform)");
    assert_eq!(details.platforms.len(), 2);
    assert!(details.platforms.contains(&"linux/amd64".to_string()));
    assert!(details.platforms.contains(&"linux/arm64".to_string()));
}

#[test]
fn test_image_details_format_pretty() {
    let details = ImageDetails::new(
        "redis:7".to_string(),
        "sha256:xyz789".to_string(),
        "OCI Image Manifest".to_string(),
        2048000,
        vec!["linux/amd64".to_string()],
        8,
        None,
    );

    let formatted = details.format_pretty();
    assert!(formatted.contains("redis:7"));
    assert!(formatted.contains("sha256:xyz789"));
    assert!(formatted.contains("OCI Image Manifest"));
    assert!(formatted.contains("MB")); // Size should be formatted
    assert!(formatted.contains("linux/amd64"));
    assert!(formatted.contains("Layers: 8"));
}

#[test]
fn test_image_details_format_pretty_multi_platform() {
    let details = ImageDetails::new(
        "postgres:15".to_string(),
        "sha256:multi123".to_string(),
        "OCI Image Index (multi-platform)".to_string(),
        10240000,
        vec![
            "linux/amd64".to_string(),
            "linux/arm64".to_string(),
            "linux/arm/v7".to_string(),
        ],
        3,
        None,
    );

    let formatted = details.format_pretty();
    assert!(formatted.contains("postgres:15"));
    assert!(formatted.contains("OCI Image Index (multi-platform)"));
    assert!(formatted.contains("Platform: 3 platforms")); // Multi-platform shows count
    assert!(formatted.contains("Layers: 3"));
}

#[test]
fn test_image_details_serialization() {
    let details = ImageDetails::new(
        "alpine:3.19".to_string(),
        "sha256:serial123".to_string(),
        "OCI Image Manifest".to_string(),
        512000,
        vec!["linux/amd64".to_string()],
        3,
        None,
    );

    let json = serde_json::to_string(&details).unwrap();
    assert!(json.contains("\"reference\":\"alpine:3.19\""));
    assert!(json.contains("\"digest\":\"sha256:serial123\""));
    assert!(json.contains("\"manifest_type\":\"OCI Image Manifest\""));
    assert!(json.contains("\"size\":512000"));
    assert!(json.contains("\"platforms\""));
    assert!(json.contains("\"layers\":3"));
}

#[test]
fn test_image_details_byte_formatting() {
    // Test bytes
    let details_bytes = ImageDetails::new(
        "tiny:latest".to_string(),
        "sha256:b1".to_string(),
        "OCI Image Manifest".to_string(),
        512,
        vec![],
        1,
        None,
    );
    let formatted = details_bytes.format_pretty();
    assert!(formatted.contains("512 B"));

    // Test KB
    let details_kb = ImageDetails::new(
        "small:latest".to_string(),
        "sha256:kb1".to_string(),
        "OCI Image Manifest".to_string(),
        2048,
        vec![],
        1,
        None,
    );
    let formatted = details_kb.format_pretty();
    assert!(formatted.contains("2.00 KB"));

    // Test MB
    let details_mb = ImageDetails::new(
        "medium:latest".to_string(),
        "sha256:mb1".to_string(),
        "OCI Image Manifest".to_string(),
        2097152,
        vec![],
        1,
        None,
    );
    let formatted = details_mb.format_pretty();
    assert!(formatted.contains("2.00 MB"));

    // Test GB
    let details_gb = ImageDetails::new(
        "large:latest".to_string(),
        "sha256:gb1".to_string(),
        "OCI Image Manifest".to_string(),
        3221225472,
        vec![],
        1,
        None,
    );
    let formatted = details_gb.format_pretty();
    assert!(formatted.contains("3.00 GB"));
}

#[test]
fn test_parse_platform_os_arch() {
    let result = parse_platform("linux/amd64");
    assert!(result.is_ok());
    let (os, arch, variant) = result.unwrap();
    assert_eq!(os, "linux");
    assert_eq!(arch, "amd64");
    assert_eq!(variant, None);
}

#[test]
fn test_parse_platform_os_arch_variant() {
    let result = parse_platform("linux/arm/v7");
    assert!(result.is_ok());
    let (os, arch, variant) = result.unwrap();
    assert_eq!(os, "linux");
    assert_eq!(arch, "arm");
    assert_eq!(variant, Some("v7".to_string()));
}

#[test]
fn test_parse_platform_invalid_single_part() {
    let result = parse_platform("linux");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Invalid platform format"));
    assert!(err.contains("os/arch"));
}

#[test]
fn test_parse_platform_invalid_too_many_parts() {
    let result = parse_platform("linux/arm/v7/extra");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Invalid platform format"));
}

#[test]
fn test_parse_platform_various_architectures() {
    let test_cases = vec![
        ("linux/arm64", "linux", "arm64", None),
        ("windows/amd64", "windows", "amd64", None),
        ("darwin/arm64", "darwin", "arm64", None),
        ("linux/386", "linux", "386", None),
    ];

    for (input, expected_os, expected_arch, expected_variant) in test_cases {
        let result = parse_platform(input);
        assert!(result.is_ok(), "Failed to parse: {}", input);
        let (os, arch, variant) = result.unwrap();
        assert_eq!(os, expected_os);
        assert_eq!(arch, expected_arch);
        assert_eq!(variant, expected_variant);
    }
}
