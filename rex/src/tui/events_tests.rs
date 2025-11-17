//! Tests for the events module.

use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Helper to create a KeyEvent for testing
fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

/// Helper to create a KeyEvent with modifiers
fn key_event_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

#[test]
fn test_event_handler_new() {
    let handler = EventHandler::new(false);
    assert!(!handler.vim_mode);

    let handler = EventHandler::new(true);
    assert!(handler.vim_mode);
}

#[test]
fn test_event_handler_default() {
    let handler = EventHandler::default();
    assert!(!handler.vim_mode);
}

// Navigation key tests

#[test]
fn test_arrow_keys_map_to_navigation() {
    let handler = EventHandler::new(false);

    assert_eq!(handler.handle_key(key_event(KeyCode::Up)), Event::Up);
    assert_eq!(handler.handle_key(key_event(KeyCode::Down)), Event::Down);
    assert_eq!(handler.handle_key(key_event(KeyCode::Left)), Event::Left);
    assert_eq!(handler.handle_key(key_event(KeyCode::Right)), Event::Right);
}

#[test]
fn test_vim_keys_when_vim_mode_disabled() {
    let handler = EventHandler::new(false);

    // Should be treated as regular characters when vim mode is off
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('k'))),
        Event::Char('k')
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('j'))),
        Event::Char('j')
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('h'))),
        Event::Char('h')
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('l'))),
        Event::Char('l')
    );
}

#[test]
fn test_vim_keys_when_vim_mode_enabled() {
    let handler = EventHandler::new(true);

    assert_eq!(handler.handle_key(key_event(KeyCode::Char('k'))), Event::Up);
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('j'))),
        Event::Down
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('h'))),
        Event::Left
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('l'))),
        Event::Right
    );
}

#[test]
fn test_arrow_keys_work_regardless_of_vim_mode() {
    let handler_no_vim = EventHandler::new(false);
    let handler_vim = EventHandler::new(true);

    // Arrow keys should work in both modes
    assert_eq!(handler_no_vim.handle_key(key_event(KeyCode::Up)), Event::Up);
    assert_eq!(handler_vim.handle_key(key_event(KeyCode::Up)), Event::Up);
}

#[test]
fn test_page_navigation_keys() {
    let handler = EventHandler::new(false);

    assert_eq!(
        handler.handle_key(key_event(KeyCode::PageUp)),
        Event::PageUp
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::PageDown)),
        Event::PageDown
    );
    assert_eq!(handler.handle_key(key_event(KeyCode::Home)), Event::Home);
    assert_eq!(handler.handle_key(key_event(KeyCode::End)), Event::End);
}

#[test]
fn test_special_navigation_keys() {
    let handler = EventHandler::new(false);

    assert_eq!(handler.handle_key(key_event(KeyCode::Enter)), Event::Enter);
    assert_eq!(handler.handle_key(key_event(KeyCode::Esc)), Event::Back);
    assert_eq!(handler.handle_key(key_event(KeyCode::Tab)), Event::Tab);
}

// Action key tests

#[test]
fn test_action_keys() {
    let handler = EventHandler::new(false);

    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('q'))),
        Event::Quit
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('/'))),
        Event::Search
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('r'))),
        Event::RegistrySelector
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('R'))),
        Event::Refresh
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('d'))),
        Event::Delete
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('y'))),
        Event::Copy
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('?'))),
        Event::Help
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('i'))),
        Event::Inspect
    );
}

#[test]
fn test_action_keys_case_sensitive() {
    let handler = EventHandler::new(false);

    // 'r' is registry selector, 'R' is refresh
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('r'))),
        Event::RegistrySelector
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('R'))),
        Event::Refresh
    );
}

#[test]
fn test_ctrl_c_maps_to_quit() {
    let handler = EventHandler::new(false);

    let ctrl_c = key_event_with_mods(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(handler.handle_key(ctrl_c), Event::Quit);
}

#[test]
fn test_regular_characters_map_to_char_event() {
    let handler = EventHandler::new(false);

    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('a'))),
        Event::Char('a')
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('z'))),
        Event::Char('z')
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char('1'))),
        Event::Char('1')
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Char(' '))),
        Event::Char(' ')
    );
}

#[test]
fn test_unknown_keys_map_to_null_char() {
    let handler = EventHandler::new(false);

    // Function keys and other special keys should be ignored
    assert_eq!(
        handler.handle_key(key_event(KeyCode::F(1))),
        Event::Char('\0')
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::F(12))),
        Event::Char('\0')
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Insert)),
        Event::Char('\0')
    );
    assert_eq!(
        handler.handle_key(key_event(KeyCode::Delete)),
        Event::Char('\0')
    );
}

// Event equality tests

#[test]
fn test_event_equality() {
    assert_eq!(Event::Up, Event::Up);
    assert_eq!(Event::Quit, Event::Quit);
    assert_eq!(Event::Char('a'), Event::Char('a'));
    assert_eq!(Event::Resize(80, 24), Event::Resize(80, 24));

    assert_ne!(Event::Up, Event::Down);
    assert_ne!(Event::Char('a'), Event::Char('b'));
    assert_ne!(Event::Resize(80, 24), Event::Resize(100, 30));
}

#[test]
fn test_event_clone() {
    let event = Event::Up;
    let cloned = event.clone();
    assert_eq!(event, cloned);

    let event = Event::Char('x');
    let cloned = event.clone();
    assert_eq!(event, cloned);
}

#[test]
fn test_event_debug() {
    // Verify Debug is implemented (for logging/debugging)
    let event = Event::Up;
    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("Up"));

    let event = Event::Resize(80, 24);
    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("Resize"));
}
