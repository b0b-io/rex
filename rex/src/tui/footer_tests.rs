//! Tests for the footer component.

use super::*;

#[test]
fn test_action_new() {
    let action = Action::new("q", "Quit");

    assert_eq!(action.key, "q");
    assert_eq!(action.description, "Quit");
    assert!(action.enabled);
}

#[test]
fn test_action_disabled() {
    let action = Action::new("d", "Delete").disabled();

    assert_eq!(action.key, "d");
    assert_eq!(action.description, "Delete");
    assert!(!action.enabled);
}

#[test]
fn test_action_enabled() {
    let action = Action::new("r", "Refresh").disabled().enabled();

    assert!(action.enabled);
}

#[test]
fn test_footer_new() {
    let actions = vec![Action::new("q", "Quit"), Action::new("r", "Refresh")];
    let footer = Footer::new(actions);

    assert_eq!(footer.actions.len(), 2);
}

#[test]
fn test_footer_with_single_action() {
    let actions = vec![Action::new("q", "Quit")];
    let footer = Footer::new(actions);

    assert_eq!(footer.actions.len(), 1);
}

#[test]
fn test_footer_with_many_actions() {
    let actions = vec![
        Action::new("Enter", "Select"),
        Action::new("/", "Search"),
        Action::new("r", "Registries"),
        Action::new("R", "Refresh"),
        Action::new("?", "Help"),
        Action::new("q", "Quit"),
    ];
    let footer = Footer::new(actions);

    assert_eq!(footer.actions.len(), 6);
}

#[test]
fn test_footer_with_mixed_enabled_disabled() {
    let actions = vec![
        Action::new("Enter", "Select"),
        Action::new("d", "Delete").disabled(),
        Action::new("q", "Quit"),
    ];
    let footer = Footer::new(actions);

    assert!(footer.actions[0].enabled);
    assert!(!footer.actions[1].enabled);
    assert!(footer.actions[2].enabled);
}

#[test]
fn test_footer_format_text_single_action() {
    let actions = vec![Action::new("q", "Quit")];
    let footer = Footer::new(actions);
    let text = footer.format_text();

    assert!(text.contains("[q]"));
    assert!(text.contains("Quit"));
}

#[test]
fn test_footer_format_text_multiple_actions() {
    let actions = vec![Action::new("Enter", "Select"), Action::new("q", "Quit")];
    let footer = Footer::new(actions);
    let text = footer.format_text();

    assert!(text.contains("[Enter]"));
    assert!(text.contains("Select"));
    assert!(text.contains("[q]"));
    assert!(text.contains("Quit"));
}

#[test]
fn test_footer_format_text_has_spacing() {
    let actions = vec![Action::new("a", "Action A"), Action::new("b", "Action B")];
    let footer = Footer::new(actions);
    let text = footer.format_text();

    // Should have spacing between actions (typically "  ")
    assert!(text.contains("Action A"));
    assert!(text.contains("Action B"));
    // Actions should not be concatenated without space
    assert!(!text.contains("Action AAction B"));
}

#[test]
fn test_action_from_tuple() {
    let action = Action::from(("q", "Quit"));

    assert_eq!(action.key, "q");
    assert_eq!(action.description, "Quit");
    assert!(action.enabled);
}

#[test]
fn test_footer_from_slice() {
    let actions = [("q", "Quit"), ("r", "Refresh")];
    let footer = Footer::from(&actions[..]);

    assert_eq!(footer.actions.len(), 2);
    assert_eq!(footer.actions[0].key, "q");
    assert_eq!(footer.actions[1].key, "r");
}
