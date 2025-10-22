use super::*;

#[test]
fn test_image_info_creation() {
    let image = ImageInfo::new("alpine".to_string(), 5, Some("2 hours ago".to_string()));

    assert_eq!(image.name, "alpine");
    assert_eq!(image.tags, 5);
    assert_eq!(image.last_updated, "2 hours ago");
}

#[test]
fn test_image_info_creation_no_timestamp() {
    let image = ImageInfo::new("nginx".to_string(), 12, None);

    assert_eq!(image.name, "nginx");
    assert_eq!(image.tags, 12);
    assert_eq!(image.last_updated, "N/A");
}

#[test]
fn test_image_info_format_pretty() {
    let image = ImageInfo::new("nginx".to_string(), 12, Some("1 day ago".to_string()));

    let formatted = image.format_pretty();
    assert!(formatted.contains("nginx"));
    assert!(formatted.contains("12"));
    assert!(formatted.contains("1 day ago"));
}

#[test]
fn test_image_info_format_pretty_no_timestamp() {
    let image = ImageInfo::new("postgres".to_string(), 8, None);

    let formatted = image.format_pretty();
    assert!(formatted.contains("postgres"));
    assert!(formatted.contains("8"));
    assert!(formatted.contains("N/A"));
}

#[test]
fn test_image_info_serialization() {
    let image = ImageInfo::new("redis".to_string(), 3, Some("3 days ago".to_string()));

    let json = serde_json::to_string(&image).unwrap();
    assert!(json.contains("\"name\":\"redis\""));
    assert!(json.contains("\"tags\":3"));
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
    let tag = TagInfo::new("latest".to_string());

    assert_eq!(tag.tag, "latest");
}

#[test]
fn test_tag_info_format_pretty() {
    let tag = TagInfo::new("v1.2.3".to_string());

    let formatted = tag.format_pretty();
    assert_eq!(formatted, "v1.2.3");
}

#[test]
fn test_tag_info_serialization() {
    let tag = TagInfo::new("3.19".to_string());

    let json = serde_json::to_string(&tag).unwrap();
    assert!(json.contains("\"tag\":\"3.19\""));
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
    );

    assert_eq!(details.reference, "alpine:latest");
    assert_eq!(details.digest, "sha256:abc123");
    assert_eq!(details.manifest_type, "OCI Image Manifest");
    assert_eq!(details.size, 1024000);
    assert_eq!(details.platforms, vec!["linux/amd64"]);
    assert_eq!(details.layers, 5);
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
    );

    let formatted = details.format_pretty();
    assert!(formatted.contains("postgres:15"));
    assert!(formatted.contains("Platforms:"));
    assert!(formatted.contains("linux/amd64"));
    assert!(formatted.contains("linux/arm64"));
    assert!(formatted.contains("linux/arm/v7"));
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
    );
    let formatted = details_gb.format_pretty();
    assert!(formatted.contains("3.00 GB"));
}
