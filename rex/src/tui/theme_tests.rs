//! Tests for the theme module.

use super::*;
use ratatui::style::{Color, Modifier};

#[test]
fn test_theme_dark_has_correct_colors() {
    let theme = Theme::dark();

    // Verify background is dark
    assert_eq!(theme.background, Color::Rgb(30, 30, 46));

    // Verify foreground is light
    assert_eq!(theme.foreground, Color::Rgb(205, 214, 244));

    // Verify semantic colors exist
    assert_eq!(theme.success, Color::Rgb(166, 227, 161));
    assert_eq!(theme.warning, Color::Rgb(249, 226, 175));
    assert_eq!(theme.error, Color::Rgb(243, 139, 168));
    assert_eq!(theme.info, Color::Rgb(137, 220, 235));
}

#[test]
fn test_theme_light_has_correct_colors() {
    let theme = Theme::light();

    // Verify background is light
    assert_eq!(theme.background, Color::Rgb(239, 241, 245));

    // Verify foreground is dark
    assert_eq!(theme.foreground, Color::Rgb(76, 79, 105));

    // Verify semantic colors exist
    assert_eq!(theme.success, Color::Rgb(64, 160, 43));
    assert_eq!(theme.warning, Color::Rgb(223, 142, 29));
    assert_eq!(theme.error, Color::Rgb(210, 15, 57));
    assert_eq!(theme.info, Color::Rgb(4, 165, 229));
}

#[test]
fn test_theme_title_style_is_bold() {
    let theme = Theme::dark();
    let style = theme.title_style();

    assert_eq!(style.fg, Some(theme.foreground));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_theme_border_style() {
    let theme = Theme::dark();
    let style = theme.border_style();

    assert_eq!(style.fg, Some(theme.border));
}

#[test]
fn test_theme_border_focused_style() {
    let theme = Theme::dark();
    let style = theme.border_focused_style();

    assert_eq!(style.fg, Some(theme.border_focused));
}

#[test]
fn test_theme_selected_style() {
    let theme = Theme::dark();
    let style = theme.selected_style();

    assert_eq!(style.bg, Some(theme.selected_bg));
    assert_eq!(style.fg, Some(theme.selected_fg));
}

#[test]
fn test_theme_success_style() {
    let theme = Theme::dark();
    let style = theme.success_style();

    assert_eq!(style.fg, Some(theme.success));
}

#[test]
fn test_theme_warning_style() {
    let theme = Theme::dark();
    let style = theme.warning_style();

    assert_eq!(style.fg, Some(theme.warning));
}

#[test]
fn test_theme_error_style() {
    let theme = Theme::dark();
    let style = theme.error_style();

    assert_eq!(style.fg, Some(theme.error));
}

#[test]
fn test_theme_info_style() {
    let theme = Theme::dark();
    let style = theme.info_style();

    assert_eq!(style.fg, Some(theme.info));
}

#[test]
fn test_theme_muted_style() {
    let theme = Theme::dark();
    let style = theme.muted_style();

    assert_eq!(style.fg, Some(theme.muted));
}

#[test]
fn test_theme_is_cloneable() {
    let theme = Theme::dark();
    let cloned = theme.clone();

    assert_eq!(theme.background, cloned.background);
    assert_eq!(theme.foreground, cloned.foreground);
}

#[test]
fn test_theme_light_and_dark_are_different() {
    let dark = Theme::dark();
    let light = Theme::light();

    // Background should be inverted
    assert_ne!(dark.background, light.background);
    assert_ne!(dark.foreground, light.foreground);
}
