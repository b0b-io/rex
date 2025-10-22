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
