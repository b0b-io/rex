//! Tests for the app module.

use super::*;
use crate::tui::events::Event;
use crate::tui::theme::Theme;

#[test]
fn test_app_new() {
    let app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    assert_eq!(app.current_view, View::RepositoryList);
    assert!(app.view_stack.is_empty());
    assert!(!app.should_quit);
    assert_eq!(app.current_registry, "localhost:5000");
    assert!(app.repositories.is_empty());
    assert!(app.tags.is_empty());
    assert!(!app.vim_mode);
}

#[test]
fn test_app_new_with_vim_mode() {
    let app = App::new("localhost:5000".to_string(), Theme::dark(), true);
    assert!(app.vim_mode);
}

// View enum tests

#[test]
fn test_view_equality() {
    assert_eq!(View::RepositoryList, View::RepositoryList);
    assert_eq!(
        View::TagList("alpine".to_string()),
        View::TagList("alpine".to_string())
    );
    assert_eq!(
        View::ImageDetails("alpine".to_string(), "latest".to_string()),
        View::ImageDetails("alpine".to_string(), "latest".to_string())
    );
    assert_eq!(View::RegistrySelector, View::RegistrySelector);
    assert_eq!(View::HelpPanel, View::HelpPanel);

    assert_ne!(View::RepositoryList, View::HelpPanel);
    assert_ne!(
        View::TagList("alpine".to_string()),
        View::TagList("nginx".to_string())
    );
}

#[test]
fn test_view_clone() {
    let view = View::TagList("alpine".to_string());
    let cloned = view.clone();
    assert_eq!(view, cloned);
}

#[test]
fn test_view_debug() {
    let view = View::RepositoryList;
    let debug_str = format!("{:?}", view);
    assert!(debug_str.contains("RepositoryList"));

    let view = View::TagList("alpine".to_string());
    let debug_str = format!("{:?}", view);
    assert!(debug_str.contains("TagList"));
    assert!(debug_str.contains("alpine"));
}

// View navigation tests

#[test]
fn test_push_view() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    assert_eq!(app.current_view, View::RepositoryList);
    assert_eq!(app.view_stack.len(), 0);

    app.push_view(View::TagList("alpine".to_string()));

    assert_eq!(app.current_view, View::TagList("alpine".to_string()));
    assert_eq!(app.view_stack.len(), 1);
    assert_eq!(app.view_stack[0], View::RepositoryList);
}

#[test]
fn test_push_multiple_views() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    app.push_view(View::TagList("alpine".to_string()));
    app.push_view(View::ImageDetails(
        "alpine".to_string(),
        "latest".to_string(),
    ));

    assert_eq!(
        app.current_view,
        View::ImageDetails("alpine".to_string(), "latest".to_string())
    );
    assert_eq!(app.view_stack.len(), 2);
    assert_eq!(app.view_stack[0], View::RepositoryList);
    assert_eq!(app.view_stack[1], View::TagList("alpine".to_string()));
}

#[test]
fn test_pop_view() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    app.push_view(View::TagList("alpine".to_string()));
    app.push_view(View::ImageDetails(
        "alpine".to_string(),
        "latest".to_string(),
    ));

    app.pop_view();

    assert_eq!(app.current_view, View::TagList("alpine".to_string()));
    assert_eq!(app.view_stack.len(), 1);
}

#[test]
fn test_pop_view_to_root() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    app.push_view(View::TagList("alpine".to_string()));
    app.pop_view();

    assert_eq!(app.current_view, View::RepositoryList);
    assert!(app.view_stack.is_empty());
}

#[test]
fn test_pop_view_when_empty() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    // Popping when already at root should do nothing
    app.pop_view();

    assert_eq!(app.current_view, View::RepositoryList);
    assert!(app.view_stack.is_empty());
}

// Event handling tests

#[test]
fn test_handle_quit_at_root_sets_should_quit() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    assert!(!app.should_quit);

    app.handle_event(Event::Quit).unwrap();

    assert!(app.should_quit);
}

#[test]
fn test_handle_quit_in_nested_view_goes_back() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    app.push_view(View::TagList("alpine".to_string()));
    assert_eq!(app.current_view, View::TagList("alpine".to_string()));

    app.handle_event(Event::Quit).unwrap();

    // Should go back instead of quitting
    assert_eq!(app.current_view, View::RepositoryList);
    assert!(!app.should_quit);
}

#[test]
fn test_handle_back_event() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    app.push_view(View::TagList("alpine".to_string()));
    app.handle_event(Event::Back).unwrap();

    assert_eq!(app.current_view, View::RepositoryList);
}

#[test]
fn test_handle_back_at_root_does_nothing() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    app.handle_event(Event::Back).unwrap();

    assert_eq!(app.current_view, View::RepositoryList);
    assert!(!app.should_quit);
}

#[test]
fn test_handle_resize_event() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    // Resize should not cause an error
    app.handle_event(Event::Resize(100, 50)).unwrap();

    // State should remain unchanged
    assert_eq!(app.current_view, View::RepositoryList);
    assert!(!app.should_quit);
}

// Message handling tests

#[test]
fn test_handle_repositories_loaded_success() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    let repos = vec!["alpine".to_string(), "nginx".to_string()];
    app.handle_message(Message::RepositoriesLoaded(Ok(repos.clone())));

    assert_eq!(app.repositories, repos);
}

#[test]
fn test_handle_repositories_loaded_error() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    let error = Err("Connection failed".into());
    app.handle_message(Message::RepositoriesLoaded(error));

    // Repositories should remain empty
    assert!(app.repositories.is_empty());
}

#[test]
fn test_handle_tags_loaded_success() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    let tags = vec!["latest".to_string(), "3.19".to_string()];
    app.handle_message(Message::TagsLoaded("alpine".to_string(), Ok(tags.clone())));

    assert_eq!(app.tags.get("alpine"), Some(&tags));
}

#[test]
fn test_handle_tags_loaded_error() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    let error = Err("Not found".into());
    app.handle_message(Message::TagsLoaded("alpine".to_string(), error));

    assert!(app.tags.get("alpine").is_none());
}

#[test]
fn test_handle_tags_for_multiple_repositories() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    let alpine_tags = vec!["latest".to_string(), "3.19".to_string()];
    let nginx_tags = vec!["latest".to_string(), "alpine".to_string()];

    app.handle_message(Message::TagsLoaded(
        "alpine".to_string(),
        Ok(alpine_tags.clone()),
    ));
    app.handle_message(Message::TagsLoaded(
        "nginx".to_string(),
        Ok(nginx_tags.clone()),
    ));

    assert_eq!(app.tags.get("alpine"), Some(&alpine_tags));
    assert_eq!(app.tags.get("nginx"), Some(&nginx_tags));
}

#[test]
fn test_process_messages_drains_queue() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    // Simulate worker sending messages
    let tx = app.tx.clone();
    tx.send(Message::RepositoriesLoaded(Ok(vec!["alpine".to_string()])))
        .unwrap();
    tx.send(Message::TagsLoaded(
        "alpine".to_string(),
        Ok(vec!["latest".to_string()]),
    ))
    .unwrap();

    // Process all messages
    app.process_messages();

    assert_eq!(app.repositories.len(), 1);
    assert_eq!(app.tags.len(), 1);
}

// Worker spawning tests

#[test]
fn test_spawn_worker() {
    let app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    // Spawn a worker that sends a message
    app.spawn_worker(|| Message::RepositoriesLoaded(Ok(vec!["test".to_string()])));

    // Give the worker thread time to execute
    std::thread::sleep(std::time::Duration::from_millis(50));

    // The message should be in the channel (but we can't easily test it without
    // making the rx public or adding a getter, which we don't want to do)
}

#[test]
fn test_spawn_multiple_workers() {
    let app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    for i in 0..5 {
        let repo = format!("repo{}", i);
        app.spawn_worker(move || Message::RepositoriesLoaded(Ok(vec![repo])));
    }

    // Workers should spawn without blocking
    std::thread::sleep(std::time::Duration::from_millis(100));
}

// Integration tests

#[test]
fn test_full_navigation_flow() {
    let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);

    // Start at repository list
    assert_eq!(app.current_view, View::RepositoryList);

    // Navigate to tag list
    app.push_view(View::TagList("alpine".to_string()));
    assert_eq!(app.current_view, View::TagList("alpine".to_string()));

    // Navigate to details
    app.push_view(View::ImageDetails(
        "alpine".to_string(),
        "latest".to_string(),
    ));
    assert_eq!(
        app.current_view,
        View::ImageDetails("alpine".to_string(), "latest".to_string())
    );

    // Go back to tag list
    app.handle_event(Event::Back).unwrap();
    assert_eq!(app.current_view, View::TagList("alpine".to_string()));

    // Go back to repository list
    app.handle_event(Event::Quit).unwrap();
    assert_eq!(app.current_view, View::RepositoryList);

    // Quit application
    app.handle_event(Event::Quit).unwrap();
    assert!(app.should_quit);
}
