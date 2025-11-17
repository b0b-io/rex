//! Tests for the title bar component.

use super::*;

#[test]
fn test_title_bar_new() {
    let title_bar = TitleBar::new();

    assert_eq!(title_bar.app_name, "Rex");
    assert_eq!(title_bar.registry_name, None);
}

#[test]
fn test_title_bar_set_registry() {
    let mut title_bar = TitleBar::new();

    title_bar.set_registry("localhost:5000".to_string());

    assert_eq!(title_bar.registry_name, Some("localhost:5000".to_string()));
}

#[test]
fn test_title_bar_clear_registry() {
    let mut title_bar = TitleBar::new();
    title_bar.set_registry("localhost:5000".to_string());

    title_bar.set_registry("".to_string());

    // Empty string should be treated as None
    assert_eq!(title_bar.registry_name, Some("".to_string()));
}

#[test]
fn test_title_bar_with_registry() {
    let title_bar = TitleBar::new().with_registry("localhost:5000".to_string());

    assert_eq!(title_bar.registry_name, Some("localhost:5000".to_string()));
}

#[test]
fn test_title_bar_format_text_without_registry() {
    let title_bar = TitleBar::new();
    let text = title_bar.format_text(80);

    // Should just be the app name
    assert!(text.starts_with("Rex"));
    assert!(!text.contains("Registry:"));
}

#[test]
fn test_title_bar_format_text_with_registry() {
    let title_bar = TitleBar::new().with_registry("localhost:5000".to_string());
    let text = title_bar.format_text(80);

    // Should have both app name and registry
    assert!(text.contains("Rex"));
    assert!(text.contains("Registry: localhost:5000"));
    assert!(text.contains("[r]"));
}

#[test]
fn test_title_bar_format_text_with_spacing() {
    let title_bar = TitleBar::new().with_registry("localhost:5000".to_string());
    let text = title_bar.format_text(80);

    let app_part = "Rex";
    let registry_part = "Registry: localhost:5000   [r]";

    // Should have spacing between left and right parts
    assert!(text.len() <= 80);
    assert!(text.starts_with(app_part));
    assert!(text.ends_with(registry_part));
}

#[test]
fn test_title_bar_format_text_narrow_terminal() {
    let title_bar = TitleBar::new().with_registry("localhost:5000".to_string());
    let text = title_bar.format_text(60);

    // Should still fit in narrow terminal
    assert!(text.len() <= 60);
    // Should at least have app name
    assert!(text.contains("Rex"));
}

#[test]
fn test_title_bar_format_text_very_narrow_terminal() {
    let title_bar = TitleBar::new().with_registry("localhost:5000".to_string());
    let text = title_bar.format_text(40);

    // Should not panic or overflow
    assert!(text.len() <= 40);
}

#[test]
fn test_title_bar_format_text_exact_fit() {
    let title_bar = TitleBar::new().with_registry("localhost:5000".to_string());

    let app_len = "Rex".len();
    let registry_len = "Registry: localhost:5000   [r]".len();
    let exact_width = app_len + registry_len;

    let text = title_bar.format_text(exact_width as u16);

    // Should fit exactly without extra spaces
    assert_eq!(text.len(), exact_width);
}
