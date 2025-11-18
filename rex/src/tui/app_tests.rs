//! Tests for the app module.

use super::*;
use crate::context::AppContext;
use crate::context::VerbosityLevel;
use crate::format::ColorChoice;
use crate::tui::events::Event;

/// Helper to create a test AppContext with default settings
fn create_test_context() -> AppContext {
    AppContext::build(ColorChoice::Never, VerbosityLevel::Normal)
}

#[test]
fn test_app_new() {
    let ctx = create_test_context();
    let app = App::new(&ctx).unwrap();

    assert_eq!(app.current_view, View::RepositoryList);
    assert!(app.view_stack.is_empty());
    assert!(!app.should_quit);
    assert!(app.repositories.is_empty());
    assert!(app.tags.is_empty());
}

#[test]
fn test_app_new_extracts_settings() {
    let ctx = create_test_context();
    let app = App::new(&ctx).unwrap();

    // Should extract vim_mode from config
    assert_eq!(app.vim_mode, ctx.config.tui.vim_mode);
    // Should have a cache directory
    assert!(!app.cache_dir.as_os_str().is_empty());
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
    let mut app = App::new(&create_test_context()).unwrap();

    assert_eq!(app.current_view, View::RepositoryList);
    assert_eq!(app.view_stack.len(), 0);

    app.push_view(View::TagList("alpine".to_string()));

    assert_eq!(app.current_view, View::TagList("alpine".to_string()));
    assert_eq!(app.view_stack.len(), 1);
    assert_eq!(app.view_stack[0], View::RepositoryList);
}

#[test]
fn test_push_multiple_views() {
    let mut app = App::new(&create_test_context()).unwrap();

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
    let mut app = App::new(&create_test_context()).unwrap();

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
    let mut app = App::new(&create_test_context()).unwrap();

    app.push_view(View::TagList("alpine".to_string()));
    app.pop_view();

    assert_eq!(app.current_view, View::RepositoryList);
    assert!(app.view_stack.is_empty());
}

#[test]
fn test_pop_view_when_empty() {
    let mut app = App::new(&create_test_context()).unwrap();

    // Popping when already at root should do nothing
    app.pop_view();

    assert_eq!(app.current_view, View::RepositoryList);
    assert!(app.view_stack.is_empty());
}

// Event handling tests

#[test]
fn test_handle_quit_at_root_sets_should_quit() {
    let mut app = App::new(&create_test_context()).unwrap();

    assert!(!app.should_quit);

    app.handle_event(Event::Quit).unwrap();

    assert!(app.should_quit);
}

#[test]
fn test_handle_quit_in_nested_view_goes_back() {
    let mut app = App::new(&create_test_context()).unwrap();

    app.push_view(View::TagList("alpine".to_string()));
    assert_eq!(app.current_view, View::TagList("alpine".to_string()));

    app.handle_event(Event::Quit).unwrap();

    // Should go back instead of quitting
    assert_eq!(app.current_view, View::RepositoryList);
    assert!(!app.should_quit);
}

#[test]
fn test_handle_back_event() {
    let mut app = App::new(&create_test_context()).unwrap();

    app.push_view(View::TagList("alpine".to_string()));
    app.handle_event(Event::Back).unwrap();

    assert_eq!(app.current_view, View::RepositoryList);
}

#[test]
fn test_handle_back_at_root_does_nothing() {
    let mut app = App::new(&create_test_context()).unwrap();

    app.handle_event(Event::Back).unwrap();

    assert_eq!(app.current_view, View::RepositoryList);
    assert!(!app.should_quit);
}

#[test]
fn test_handle_resize_event() {
    let mut app = App::new(&create_test_context()).unwrap();

    // Resize should not cause an error
    app.handle_event(Event::Resize(100, 50)).unwrap();

    // State should remain unchanged
    assert_eq!(app.current_view, View::RepositoryList);
    assert!(!app.should_quit);
}

// Message handling tests

#[test]
fn test_handle_repositories_loaded_success() {
    let mut app = App::new(&create_test_context()).unwrap();

    let repos = vec!["alpine".to_string(), "nginx".to_string()];
    app.handle_message(Message::RepositoriesLoaded(Ok(repos.clone())));

    assert_eq!(app.repositories, repos);
}

#[test]
fn test_handle_repositories_loaded_error() {
    let mut app = App::new(&create_test_context()).unwrap();

    let error = Err("Connection failed".into());
    app.handle_message(Message::RepositoriesLoaded(error));

    // Repositories should remain empty
    assert!(app.repositories.is_empty());
}

#[test]
fn test_handle_tags_loaded_success() {
    let mut app = App::new(&create_test_context()).unwrap();

    let tags = vec!["latest".to_string(), "3.19".to_string()];
    app.handle_message(Message::TagsLoaded("alpine".to_string(), Ok(tags.clone())));

    assert_eq!(app.tags.get("alpine"), Some(&tags));
}

#[test]
fn test_handle_tags_loaded_error() {
    let mut app = App::new(&create_test_context()).unwrap();

    let error = Err("Not found".into());
    app.handle_message(Message::TagsLoaded("alpine".to_string(), error));

    assert!(app.tags.get("alpine").is_none());
}

#[test]
fn test_handle_tags_for_multiple_repositories() {
    let mut app = App::new(&create_test_context()).unwrap();

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
    let mut app = App::new(&create_test_context()).unwrap();

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
    let app = App::new(&create_test_context()).unwrap();

    // Spawn a worker that sends a message
    app.spawn_worker(|| Message::RepositoriesLoaded(Ok(vec!["test".to_string()])));

    // Give the worker thread time to execute
    std::thread::sleep(std::time::Duration::from_millis(50));

    // The message should be in the channel (but we can't easily test it without
    // making the rx public or adding a getter, which we don't want to do)
}

#[test]
fn test_spawn_multiple_workers() {
    let app = App::new(&create_test_context()).unwrap();

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
    let mut app = App::new(&create_test_context()).unwrap();

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

// Repository list integration tests

#[test]
fn test_repo_list_state_initialized() {
    let app = App::new(&create_test_context()).unwrap();

    assert_eq!(app.repo_list_state.items.len(), 0);
    assert_eq!(app.repo_list_state.selected, 0);
    assert!(!app.repo_list_state.loading);
}

#[test]
fn test_handle_repositories_loaded_populates_repo_list_state() {
    let mut app = App::new(&create_test_context()).unwrap();
    app.repo_list_state.loading = true;

    let repos = vec!["alpine".to_string(), "nginx".to_string()];
    app.handle_message(Message::RepositoriesLoaded(Ok(repos.clone())));

    assert_eq!(app.repo_list_state.items.len(), 2);
    assert_eq!(app.repo_list_state.items[0].name, "alpine");
    assert_eq!(app.repo_list_state.items[1].name, "nginx");
    assert!(!app.repo_list_state.loading);
}

#[test]
fn test_handle_repositories_loaded_error_clears_loading() {
    let mut app = App::new(&create_test_context()).unwrap();
    app.repo_list_state.loading = true;

    app.handle_message(Message::RepositoriesLoaded(Err("error".into())));

    assert!(!app.repo_list_state.loading);
    assert_eq!(app.repo_list_state.items.len(), 0);
}

#[test]
fn test_repo_list_up_event() {
    let mut app = App::new(&create_test_context()).unwrap();
    let repos = vec!["alpine".to_string(), "nginx".to_string()];
    app.handle_message(Message::RepositoriesLoaded(Ok(repos)));

    // Start at first item
    assert_eq!(app.repo_list_state.selected, 0);

    // Navigate down
    app.handle_event(Event::Down).unwrap();
    assert_eq!(app.repo_list_state.selected, 1);

    // Navigate back up
    app.handle_event(Event::Up).unwrap();
    assert_eq!(app.repo_list_state.selected, 0);
}

#[test]
fn test_repo_list_down_event() {
    let mut app = App::new(&create_test_context()).unwrap();
    let repos = vec![
        "alpine".to_string(),
        "nginx".to_string(),
        "redis".to_string(),
    ];
    app.handle_message(Message::RepositoriesLoaded(Ok(repos)));

    assert_eq!(app.repo_list_state.selected, 0);

    app.handle_event(Event::Down).unwrap();
    assert_eq!(app.repo_list_state.selected, 1);

    app.handle_event(Event::Down).unwrap();
    assert_eq!(app.repo_list_state.selected, 2);
}

#[test]
fn test_repo_list_enter_navigates_to_tag_list() {
    let mut app = App::new(&create_test_context()).unwrap();
    let repos = vec!["alpine".to_string(), "nginx".to_string()];
    app.handle_message(Message::RepositoriesLoaded(Ok(repos)));

    // Select first repository and press Enter
    app.handle_event(Event::Enter).unwrap();

    // Should navigate to tag list for alpine
    assert_eq!(app.current_view, View::TagList("alpine".to_string()));
    assert_eq!(app.view_stack.len(), 1);
    assert_eq!(app.view_stack[0], View::RepositoryList);
}

#[test]
fn test_repo_list_enter_on_empty_list_does_nothing() {
    let mut app = App::new(&create_test_context()).unwrap();

    // No repositories loaded
    app.handle_event(Event::Enter).unwrap();

    // Should stay on repository list
    assert_eq!(app.current_view, View::RepositoryList);
    assert_eq!(app.view_stack.len(), 0);
}

#[test]
fn test_load_repositories_sets_loading_state() {
    let mut app = App::new(&create_test_context()).unwrap();

    assert!(!app.repo_list_state.loading);

    app.load_repositories();

    assert!(app.repo_list_state.loading);
}

#[test]
fn test_repo_list_refresh_event() {
    let mut app = App::new(&create_test_context()).unwrap();

    app.handle_event(Event::Refresh).unwrap();

    assert!(app.repo_list_state.loading);
}
