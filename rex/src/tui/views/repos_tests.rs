//! Tests for the repository list view data model.

use super::*;

#[test]
fn test_repository_list_state_new() {
    let state = RepositoryListState::new();

    assert_eq!(state.items.len(), 0);
    assert_eq!(state.selected, 0);
    assert_eq!(state.scroll_offset, 0);
    assert!(!state.loading);
    assert_eq!(state.filter, "");
}

#[test]
fn test_select_next_moves_down() {
    let mut state = RepositoryListState::new();
    state.items = vec![
        RepositoryItem {
            name: "alpine".to_string(),
            tag_count: 5,
            total_size: 1024,
            last_updated: None,
        },
        RepositoryItem {
            name: "nginx".to_string(),
            tag_count: 10,
            total_size: 2048,
            last_updated: None,
        },
    ];

    assert_eq!(state.selected, 0);
    state.select_next();
    assert_eq!(state.selected, 1);
}

#[test]
fn test_select_next_at_end_stays_at_end() {
    let mut state = RepositoryListState::new();
    state.items = vec![RepositoryItem {
        name: "alpine".to_string(),
        tag_count: 5,
        total_size: 1024,
        last_updated: None,
    }];
    state.selected = 0;

    state.select_next();
    // Should stay at 0 since we're at the last item
    assert_eq!(state.selected, 0);
}

#[test]
fn test_select_previous_moves_up() {
    let mut state = RepositoryListState::new();
    state.items = vec![
        RepositoryItem {
            name: "alpine".to_string(),
            tag_count: 5,
            total_size: 1024,
            last_updated: None,
        },
        RepositoryItem {
            name: "nginx".to_string(),
            tag_count: 10,
            total_size: 2048,
            last_updated: None,
        },
    ];
    state.selected = 1;

    state.select_previous();
    assert_eq!(state.selected, 0);
}

#[test]
fn test_select_previous_at_start_stays_at_start() {
    let mut state = RepositoryListState::new();
    state.items = vec![RepositoryItem {
        name: "alpine".to_string(),
        tag_count: 5,
        total_size: 1024,
        last_updated: None,
    }];
    state.selected = 0;

    state.select_previous();
    assert_eq!(state.selected, 0);
}

#[test]
fn test_selected_item_returns_current_item() {
    let mut state = RepositoryListState::new();
    state.items = vec![
        RepositoryItem {
            name: "alpine".to_string(),
            tag_count: 5,
            total_size: 1024,
            last_updated: None,
        },
        RepositoryItem {
            name: "nginx".to_string(),
            tag_count: 10,
            total_size: 2048,
            last_updated: None,
        },
    ];
    state.selected = 1;

    let item = state.selected_item();
    assert!(item.is_some());
    assert_eq!(item.unwrap().name, "nginx");
}

#[test]
fn test_selected_item_returns_none_when_empty() {
    let state = RepositoryListState::new();
    assert!(state.selected_item().is_none());
}

#[test]
fn test_filtered_items_returns_all_when_no_filter() {
    let mut state = RepositoryListState::new();
    state.items = vec![
        RepositoryItem {
            name: "alpine".to_string(),
            tag_count: 5,
            total_size: 1024,
            last_updated: None,
        },
        RepositoryItem {
            name: "nginx".to_string(),
            tag_count: 10,
            total_size: 2048,
            last_updated: None,
        },
    ];

    let filtered = state.filtered_items();
    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_filtered_items_filters_by_name() {
    let mut state = RepositoryListState::new();
    state.items = vec![
        RepositoryItem {
            name: "alpine".to_string(),
            tag_count: 5,
            total_size: 1024,
            last_updated: None,
        },
        RepositoryItem {
            name: "nginx".to_string(),
            tag_count: 10,
            total_size: 2048,
            last_updated: None,
        },
        RepositoryItem {
            name: "redis".to_string(),
            tag_count: 8,
            total_size: 3072,
            last_updated: None,
        },
    ];
    state.filter = "ng".to_string();

    let filtered = state.filtered_items();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "nginx");
}

#[test]
fn test_filtered_items_case_sensitive() {
    let mut state = RepositoryListState::new();
    state.items = vec![RepositoryItem {
        name: "Alpine".to_string(),
        tag_count: 5,
        total_size: 1024,
        last_updated: None,
    }];
    state.filter = "alp".to_string();

    let filtered = state.filtered_items();
    // Case-sensitive: "alp" does not match "Alpine"
    assert_eq!(filtered.len(), 0);
}
