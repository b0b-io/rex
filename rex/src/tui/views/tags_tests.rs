//! Tests for the tag list view data model.

use super::*;

#[test]
fn test_tag_list_state_new() {
    let state = TagListState::new("alpine".to_string());

    assert_eq!(state.repository, "alpine");
    assert_eq!(state.items.len(), 0);
    assert_eq!(state.selected, 0);
    assert_eq!(state.scroll_offset, 0);
    assert!(!state.loading);
}

#[test]
fn test_tag_list_state_default() {
    let state = TagListState::default();

    assert_eq!(state.repository, "");
    assert_eq!(state.items.len(), 0);
}

#[test]
fn test_select_next_moves_down() {
    let mut state = TagListState::new("alpine".to_string());
    state.items = vec![
        TagItem::new(
            "latest".to_string(),
            "sha256:abc123".to_string(),
            1024,
            None,
            vec!["linux/amd64".to_string()],
        ),
        TagItem::new(
            "3.19".to_string(),
            "sha256:def456".to_string(),
            2048,
            None,
            vec!["linux/amd64".to_string()],
        ),
    ];

    assert_eq!(state.selected, 0);
    state.select_next();
    assert_eq!(state.selected, 1);
}

#[test]
fn test_select_next_at_end_stays_at_end() {
    let mut state = TagListState::new("alpine".to_string());
    state.items = vec![TagItem::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        None,
        vec!["linux/amd64".to_string()],
    )];
    state.selected = 0;

    state.select_next();
    // Should stay at 0 since we're at the last item
    assert_eq!(state.selected, 0);
}

#[test]
fn test_select_previous_moves_up() {
    let mut state = TagListState::new("alpine".to_string());
    state.items = vec![
        TagItem::new(
            "latest".to_string(),
            "sha256:abc123".to_string(),
            1024,
            None,
            vec!["linux/amd64".to_string()],
        ),
        TagItem::new(
            "3.19".to_string(),
            "sha256:def456".to_string(),
            2048,
            None,
            vec!["linux/amd64".to_string()],
        ),
    ];
    state.selected = 1;

    state.select_previous();
    assert_eq!(state.selected, 0);
}

#[test]
fn test_select_previous_at_start_stays_at_start() {
    let mut state = TagListState::new("alpine".to_string());
    state.items = vec![TagItem::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        None,
        vec!["linux/amd64".to_string()],
    )];
    state.selected = 0;

    state.select_previous();
    assert_eq!(state.selected, 0);
}

#[test]
fn test_selected_item_returns_current_item() {
    let mut state = TagListState::new("alpine".to_string());
    state.items = vec![
        TagItem::new(
            "latest".to_string(),
            "sha256:abc123".to_string(),
            1024,
            None,
            vec!["linux/amd64".to_string()],
        ),
        TagItem::new(
            "3.19".to_string(),
            "sha256:def456".to_string(),
            2048,
            None,
            vec!["linux/amd64".to_string()],
        ),
    ];
    state.selected = 1;

    let item = state.selected_item();
    assert!(item.is_some());
    assert_eq!(item.unwrap().tag, "3.19");
}

#[test]
fn test_selected_item_returns_none_when_empty() {
    let state = TagListState::new("alpine".to_string());
    assert!(state.selected_item().is_none());
}

#[test]
fn test_tag_item_equality() {
    let item1 = TagItem::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        None,
        vec!["linux/amd64".to_string()],
    );

    let item2 = TagItem::new(
        "latest".to_string(),
        "sha256:abc123".to_string(),
        1024,
        None,
        vec!["linux/amd64".to_string()],
    );

    assert_eq!(item1, item2);
}

#[test]
fn test_tag_item_with_multiple_platforms() {
    let item = TagItem::new(
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

    // TagInfo formats platforms as a string, so check that
    assert!(item.platforms.contains("platforms")); // Should say "3 platforms"
}

#[test]
fn test_navigation_with_multiple_items() {
    let mut state = TagListState::new("alpine".to_string());
    state.items = vec![
        TagItem::new(
            "3.19".to_string(),
            "sha256:aaa".to_string(),
            1024,
            None,
            vec![],
        ),
        TagItem::new(
            "3.18".to_string(),
            "sha256:bbb".to_string(),
            2048,
            None,
            vec![],
        ),
        TagItem::new(
            "3.17".to_string(),
            "sha256:ccc".to_string(),
            3072,
            None,
            vec![],
        ),
    ];

    // Start at first item
    assert_eq!(state.selected, 0);
    assert_eq!(state.selected_item().unwrap().tag, "3.19");

    // Move down
    state.select_next();
    assert_eq!(state.selected, 1);
    assert_eq!(state.selected_item().unwrap().tag, "3.18");

    state.select_next();
    assert_eq!(state.selected, 2);
    assert_eq!(state.selected_item().unwrap().tag, "3.17");

    // Try to move past end
    state.select_next();
    assert_eq!(state.selected, 2); // Should stay at end

    // Move back up
    state.select_previous();
    assert_eq!(state.selected, 1);
    assert_eq!(state.selected_item().unwrap().tag, "3.18");
}
