//! Tests for the image details view.

use super::*;

#[test]
fn test_image_details_state_new() {
    let state = ImageDetailsState::new("alpine".to_string(), "latest".to_string());

    assert_eq!(state.repository, "alpine");
    assert_eq!(state.tag, "latest");
    assert!(state.manifest.is_none());
    assert!(state.config.is_none());
    assert_eq!(state.scroll_offset, 0);
    assert!(!state.loading);
}

#[test]
fn test_scroll_down() {
    let mut state = ImageDetailsState::new("alpine".to_string(), "latest".to_string());

    assert_eq!(state.scroll_offset, 0);
    state.scroll_down();
    assert_eq!(state.scroll_offset, 1);
    state.scroll_down();
    assert_eq!(state.scroll_offset, 2);
}

#[test]
fn test_scroll_up() {
    let mut state = ImageDetailsState::new("alpine".to_string(), "latest".to_string());
    state.scroll_offset = 5;

    state.scroll_up();
    assert_eq!(state.scroll_offset, 4);
    state.scroll_up();
    assert_eq!(state.scroll_offset, 3);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = ImageDetailsState::new("alpine".to_string(), "latest".to_string());

    state.scroll_up();
    assert_eq!(state.scroll_offset, 0); // Should not go negative
}

#[test]
fn test_scroll_page_down() {
    let mut state = ImageDetailsState::new("alpine".to_string(), "latest".to_string());

    state.scroll_page_down();
    assert_eq!(state.scroll_offset, 10);
    state.scroll_page_down();
    assert_eq!(state.scroll_offset, 20);
}

#[test]
fn test_scroll_page_up() {
    let mut state = ImageDetailsState::new("alpine".to_string(), "latest".to_string());
    state.scroll_offset = 25;

    state.scroll_page_up();
    assert_eq!(state.scroll_offset, 15);
    state.scroll_page_up();
    assert_eq!(state.scroll_offset, 5);
}

#[test]
fn test_scroll_to_top() {
    let mut state = ImageDetailsState::new("alpine".to_string(), "latest".to_string());
    state.scroll_offset = 100;

    state.scroll_to_top();
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn test_default() {
    let state = ImageDetailsState::default();

    assert_eq!(state.repository, "");
    assert_eq!(state.tag, "");
    assert!(state.manifest.is_none());
    assert_eq!(state.scroll_offset, 0);
}
