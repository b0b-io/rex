//! Tests for the shell module.

use super::*;
use ratatui::layout::Rect;

#[test]
fn test_shell_layout_with_all_components() {
    let area = Rect::new(0, 0, 80, 24);
    let layout = ShellLayout::calculate(area, true, true);

    // Title bar: 3 lines (border + content + border)
    assert_eq!(layout.title_bar.height, 3);
    assert_eq!(layout.title_bar.y, 0);

    // Context bar: 1 line
    assert!(layout.context_bar.is_some());
    assert_eq!(layout.context_bar.unwrap().height, 1);
    assert_eq!(layout.context_bar.unwrap().y, 3);

    // Content: fills remaining space
    assert_eq!(layout.content.y, 4);
    assert!(layout.content.height > 10); // Should have substantial height

    // Status line: 1 line
    assert!(layout.status_line.is_some());
    assert_eq!(layout.status_line.unwrap().height, 1);

    // Footer: 3 lines (border + content + border)
    assert_eq!(layout.footer.height, 3);
    assert_eq!(layout.footer.y, 21); // 24 - 3 = 21
}

#[test]
fn test_shell_layout_without_optional_components() {
    let area = Rect::new(0, 0, 80, 24);
    let layout = ShellLayout::calculate(area, false, false);

    // Title bar
    assert_eq!(layout.title_bar.height, 3);
    assert_eq!(layout.title_bar.y, 0);

    // No context bar
    assert!(layout.context_bar.is_none());

    // Content starts right after title bar
    assert_eq!(layout.content.y, 3);

    // No status line
    assert!(layout.status_line.is_none());

    // Footer at bottom
    assert_eq!(layout.footer.height, 3);
    assert_eq!(layout.footer.y, 21);

    // Content should be larger without optional components
    assert!(layout.content.height > 12);
}

#[test]
fn test_shell_layout_with_context_only() {
    let area = Rect::new(0, 0, 80, 24);
    let layout = ShellLayout::calculate(area, true, false);

    assert!(layout.context_bar.is_some());
    assert!(layout.status_line.is_none());
    assert_eq!(layout.content.y, 4); // After title + context
}

#[test]
fn test_shell_layout_with_status_only() {
    let area = Rect::new(0, 0, 80, 24);
    let layout = ShellLayout::calculate(area, false, true);

    assert!(layout.context_bar.is_none());
    assert!(layout.status_line.is_some());
    assert_eq!(layout.content.y, 3); // After title only
}

#[test]
fn test_shell_layout_small_terminal() {
    let area = Rect::new(0, 0, 60, 15); // Minimum viable size
    let layout = ShellLayout::calculate(area, false, false);

    // All components should fit
    assert_eq!(layout.title_bar.height, 3);
    assert_eq!(layout.footer.height, 3);
    assert!(layout.content.height >= 9); // 15 - 3 - 3 = 9
}

#[test]
fn test_shell_layout_large_terminal() {
    let area = Rect::new(0, 0, 120, 40);
    let layout = ShellLayout::calculate(area, true, true);

    // Fixed components stay same size
    assert_eq!(layout.title_bar.height, 3);
    assert_eq!(layout.footer.height, 3);
    assert_eq!(layout.context_bar.unwrap().height, 1);
    assert_eq!(layout.status_line.unwrap().height, 1);

    // Content expands
    assert!(layout.content.height > 25);
}

#[test]
fn test_shell_layout_full_width() {
    let area = Rect::new(0, 0, 80, 24);
    let layout = ShellLayout::calculate(area, false, false);

    // All areas should span full width
    assert_eq!(layout.title_bar.width, 80);
    assert_eq!(layout.content.width, 80);
    assert_eq!(layout.footer.width, 80);
}

#[test]
fn test_shell_layout_areas_dont_overlap() {
    let area = Rect::new(0, 0, 80, 24);
    let layout = ShellLayout::calculate(area, true, true);

    // Verify no overlapping Y coordinates
    let title_end = layout.title_bar.y + layout.title_bar.height;
    assert_eq!(layout.context_bar.unwrap().y, title_end);

    let context_end = layout.context_bar.unwrap().y + layout.context_bar.unwrap().height;
    assert_eq!(layout.content.y, context_end);

    let content_end = layout.content.y + layout.content.height;
    assert_eq!(layout.status_line.unwrap().y, content_end);

    let status_end = layout.status_line.unwrap().y + layout.status_line.unwrap().height;
    assert_eq!(layout.footer.y, status_end);
}

#[test]
fn test_shell_layout_height_adds_up() {
    let area = Rect::new(0, 0, 80, 24);
    let layout = ShellLayout::calculate(area, true, true);

    let total_height = layout.title_bar.height
        + layout.context_bar.unwrap().height
        + layout.content.height
        + layout.status_line.unwrap().height
        + layout.footer.height;

    assert_eq!(total_height, 24);
}
