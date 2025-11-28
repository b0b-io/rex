//! Tests for banner functionality.

use super::*;
use std::time::Duration;

#[test]
fn test_banner_creation() {
    let banner = Banner::new(1, "Test message".to_string(), BannerType::Error);

    assert_eq!(banner.id(), 1);
    assert_eq!(banner.banner_type(), BannerType::Error);
}

#[test]
fn test_banner_type_symbols() {
    let loading = Banner::new(1, "Loading".to_string(), BannerType::Loading);
    let warning = Banner::new(2, "Warning".to_string(), BannerType::Warning);
    let error = Banner::new(3, "Error".to_string(), BannerType::Error);
    let success = Banner::new(4, "Success".to_string(), BannerType::Success);
    let info = Banner::new(5, "Info".to_string(), BannerType::Info);

    assert_eq!(loading.symbol(), "⟳");
    assert_eq!(warning.symbol(), "⚠");
    assert_eq!(error.symbol(), "✗");
    assert_eq!(success.symbol(), "✓");
    assert_eq!(info.symbol(), "ℹ");
}

#[test]
fn test_dismissible_banners() {
    let loading = Banner::new(1, "Loading".to_string(), BannerType::Loading);
    let warning = Banner::new(2, "Warning".to_string(), BannerType::Warning);
    let error = Banner::new(3, "Error".to_string(), BannerType::Error);
    let success = Banner::new(4, "Success".to_string(), BannerType::Success);
    let info = Banner::new(5, "Info".to_string(), BannerType::Info);

    assert!(!loading.dismissible);
    assert!(warning.dismissible);
    assert!(error.dismissible);
    assert!(!success.dismissible);
    assert!(info.dismissible);
}

#[test]
fn test_banner_auto_dismiss() {
    let banner = Banner::new(1, "Success".to_string(), BannerType::Success);

    // Should not auto-dismiss immediately
    assert!(!banner.should_auto_dismiss());

    // Create a banner with a timestamp in the past (simulate 6 seconds ago)
    let mut old_banner = Banner::new(2, "Old success".to_string(), BannerType::Success);
    old_banner.created_at = Instant::now() - Duration::from_secs(6);

    // Should auto-dismiss after 5 seconds
    assert!(old_banner.should_auto_dismiss());

    // Other banner types should not auto-dismiss
    let mut old_error = Banner::new(3, "Old error".to_string(), BannerType::Error);
    old_error.created_at = Instant::now() - Duration::from_secs(10);
    assert!(!old_error.should_auto_dismiss());
}

#[test]
fn test_banner_manager_creation() {
    let manager = BannerManager::new();

    assert!(!manager.has_banners());
    assert_eq!(manager.count(), 0);
}

#[test]
fn test_banner_manager_add() {
    let mut manager = BannerManager::new();

    let id1 = manager.add("First banner".to_string(), BannerType::Info);
    assert_eq!(id1, 0);
    assert!(manager.has_banners());
    assert_eq!(manager.count(), 1);

    let id2 = manager.add("Second banner".to_string(), BannerType::Warning);
    assert_eq!(id2, 1);
    assert_eq!(manager.count(), 2);
}

#[test]
fn test_banner_manager_remove() {
    let mut manager = BannerManager::new();

    let id1 = manager.add("First".to_string(), BannerType::Info);
    let id2 = manager.add("Second".to_string(), BannerType::Warning);

    assert_eq!(manager.count(), 2);

    manager.remove(id1);
    assert_eq!(manager.count(), 1);

    manager.remove(id2);
    assert_eq!(manager.count(), 0);
    assert!(!manager.has_banners());
}

#[test]
fn test_banner_manager_remove_type() {
    let mut manager = BannerManager::new();

    manager.add("Loading 1".to_string(), BannerType::Loading);
    manager.add("Error 1".to_string(), BannerType::Error);
    manager.add("Loading 2".to_string(), BannerType::Loading);
    manager.add("Warning 1".to_string(), BannerType::Warning);

    assert_eq!(manager.count(), 4);

    manager.remove_type(BannerType::Loading);
    assert_eq!(manager.count(), 2);

    manager.remove_type(BannerType::Error);
    assert_eq!(manager.count(), 1);
}

#[test]
fn test_banner_manager_clear() {
    let mut manager = BannerManager::new();

    manager.add("First".to_string(), BannerType::Info);
    manager.add("Second".to_string(), BannerType::Warning);
    manager.add("Third".to_string(), BannerType::Error);

    assert_eq!(manager.count(), 3);

    manager.clear();
    assert_eq!(manager.count(), 0);
    assert!(!manager.has_banners());
}

#[test]
fn test_banner_manager_auto_dismiss() {
    let mut manager = BannerManager::new();

    // Add a success banner (should auto-dismiss after 5s)
    manager.add("Success".to_string(), BannerType::Success);

    // Add an error banner (should not auto-dismiss)
    manager.add("Error".to_string(), BannerType::Error);

    assert_eq!(manager.count(), 2);

    // Immediately processing should not remove anything
    manager.process_auto_dismiss();
    assert_eq!(manager.count(), 2);

    // Manually age the success banner
    if let Some(banner) = manager.banners.first_mut() {
        banner.created_at = Instant::now() - Duration::from_secs(6);
    }

    // Now it should be removed
    manager.process_auto_dismiss();
    assert_eq!(manager.count(), 1);

    // Verify the error banner remains
    assert_eq!(manager.banners[0].banner_type(), BannerType::Error);
}

#[test]
fn test_banner_manager_dismiss_latest() {
    let mut manager = BannerManager::new();

    // Add various banners
    manager.add("Loading".to_string(), BannerType::Loading); // Not dismissible
    manager.add("Error".to_string(), BannerType::Error); // Dismissible
    manager.add("Success".to_string(), BannerType::Success); // Not dismissible
    manager.add("Warning".to_string(), BannerType::Warning); // Dismissible

    assert_eq!(manager.count(), 4);

    // Dismiss latest dismissible (Warning)
    manager.dismiss_latest();
    assert_eq!(manager.count(), 3);

    // Dismiss next latest dismissible (Error)
    manager.dismiss_latest();
    assert_eq!(manager.count(), 2);

    // Now only non-dismissible banners remain
    manager.dismiss_latest();
    assert_eq!(manager.count(), 2); // No change

    // Verify remaining banners are not dismissible
    assert_eq!(manager.banners[0].banner_type(), BannerType::Loading);
    assert_eq!(manager.banners[1].banner_type(), BannerType::Success);
}

#[test]
fn test_banner_manager_default() {
    let manager = BannerManager::default();

    assert!(!manager.has_banners());
    assert_eq!(manager.count(), 0);
}
