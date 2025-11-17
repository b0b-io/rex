//! Application state management for Rex TUI.
//!
//! Handles the application state, view navigation, and message passing between
//! the UI thread and background workers.

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};

use super::Result;
use super::events::Event;
use super::theme::Theme;

/// Views in the application.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // TODO: Remove when integrated into main TUI loop
pub enum View {
    /// List of repositories in the registry
    RepositoryList,
    /// List of tags for a specific repository
    TagList(String),
    /// Detailed view of a specific image (repository + tag)
    ImageDetails(String, String),
    /// Registry selector modal
    RegistrySelector,
    /// Help panel overlay
    HelpPanel,
}

/// Messages sent from background workers to the UI thread.
#[derive(Debug)]
#[allow(dead_code)] // TODO: Remove when workers are implemented
pub enum Message {
    /// Repositories loaded successfully or with error
    RepositoriesLoaded(Result<Vec<String>>),
    /// Tags loaded for a repository
    TagsLoaded(String, Result<Vec<String>>),
    /// Manifest loaded for an image
    ManifestLoaded(String, String, Result<Vec<u8>>),
    /// Generic error message
    Error(String),
}

/// Application state.
#[allow(dead_code)] // TODO: Remove when integrated into main TUI loop
pub struct App {
    // State
    /// Current active view
    pub current_view: View,
    /// Stack of previous views for back navigation
    pub view_stack: Vec<View>,
    /// Whether the application should quit
    pub should_quit: bool,

    // Registry
    /// Current registry URL
    pub current_registry: String,

    // Data (cached)
    /// List of repositories
    pub repositories: Vec<String>,
    /// Tags for each repository (keyed by repository name)
    pub tags: HashMap<String, Vec<String>>,

    // Communication
    /// Sender for messages from workers
    tx: Sender<Message>,
    /// Receiver for messages from workers
    rx: Receiver<Message>,

    // Config
    /// Theme for styling
    pub theme: Theme,
    /// Whether vim mode is enabled
    pub vim_mode: bool,
}

#[allow(dead_code)] // TODO: Remove when integrated into main TUI loop
impl App {
    /// Create a new application state.
    ///
    /// # Arguments
    ///
    /// * `registry` - The registry URL to connect to
    /// * `theme` - The theme to use for styling
    /// * `vim_mode` - Whether vim mode navigation is enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::app::App;
    /// use rex::tui::theme::Theme;
    ///
    /// let app = App::new("localhost:5000".to_string(), Theme::dark(), false);
    /// ```
    pub fn new(registry: String, theme: Theme, vim_mode: bool) -> Self {
        let (tx, rx) = channel();

        Self {
            current_view: View::RepositoryList,
            view_stack: vec![],
            should_quit: false,
            current_registry: registry,
            repositories: vec![],
            tags: HashMap::new(),
            tx,
            rx,
            theme,
            vim_mode,
        }
    }

    /// Handle an event from the user.
    ///
    /// Routes the event to the appropriate handler based on the current view.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Errors
    ///
    /// Returns an error if event handling fails.
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Quit => {
                if self.view_stack.is_empty() {
                    // At root level, quit the application
                    self.should_quit = true;
                } else {
                    // In a nested view, go back
                    self.pop_view();
                }
            }
            Event::Back => {
                self.pop_view();
            }
            Event::Resize(_, _) => {
                // Terminal will redraw automatically
            }
            _ => {
                // Delegate to view-specific event handler
                self.handle_view_event(event)?;
            }
        }
        Ok(())
    }

    /// Handle a view-specific event.
    ///
    /// Routes events to the appropriate handler based on the current view.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Errors
    ///
    /// Returns an error if event handling fails.
    fn handle_view_event(&mut self, event: Event) -> Result<()> {
        match &self.current_view {
            View::RepositoryList => self.handle_repo_list_event(event),
            View::TagList(_) => self.handle_tag_list_event(event),
            View::ImageDetails(_, _) => self.handle_details_event(event),
            View::RegistrySelector => self.handle_registry_selector_event(event),
            View::HelpPanel => self.handle_help_event(event),
        }
    }

    /// Handle events in the repository list view.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Errors
    ///
    /// Returns an error if event handling fails.
    fn handle_repo_list_event(&mut self, _event: Event) -> Result<()> {
        // TODO: Implement when repository list view is added
        Ok(())
    }

    /// Handle events in the tag list view.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Errors
    ///
    /// Returns an error if event handling fails.
    fn handle_tag_list_event(&mut self, _event: Event) -> Result<()> {
        // TODO: Implement when tag list view is added
        Ok(())
    }

    /// Handle events in the image details view.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Errors
    ///
    /// Returns an error if event handling fails.
    fn handle_details_event(&mut self, _event: Event) -> Result<()> {
        // TODO: Implement when details view is added
        Ok(())
    }

    /// Handle events in the registry selector view.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Errors
    ///
    /// Returns an error if event handling fails.
    fn handle_registry_selector_event(&mut self, _event: Event) -> Result<()> {
        // TODO: Implement when registry selector is added
        Ok(())
    }

    /// Handle events in the help panel view.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Errors
    ///
    /// Returns an error if event handling fails.
    fn handle_help_event(&mut self, _event: Event) -> Result<()> {
        // TODO: Implement when help panel is added
        Ok(())
    }

    /// Push a new view onto the stack and navigate to it.
    ///
    /// # Arguments
    ///
    /// * `view` - The view to navigate to
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::app::{App, View};
    /// use rex::tui::theme::Theme;
    ///
    /// let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);
    /// app.push_view(View::TagList("alpine".to_string()));
    /// ```
    pub fn push_view(&mut self, view: View) {
        self.view_stack.push(self.current_view.clone());
        self.current_view = view;
    }

    /// Pop the current view and return to the previous one.
    ///
    /// If there are no previous views, this does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::app::{App, View};
    /// use rex::tui::theme::Theme;
    ///
    /// let mut app = App::new("localhost:5000".to_string(), Theme::dark(), false);
    /// app.push_view(View::TagList("alpine".to_string()));
    /// app.pop_view();
    /// assert_eq!(app.current_view, View::RepositoryList);
    /// ```
    pub fn pop_view(&mut self) {
        if let Some(view) = self.view_stack.pop() {
            self.current_view = view;
        }
    }

    /// Process all pending messages from background workers.
    ///
    /// This should be called regularly in the main loop to handle
    /// asynchronous updates from worker threads.
    pub fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            self.handle_message(msg);
        }
    }

    /// Handle a message from a background worker.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to handle
    fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::RepositoriesLoaded(Ok(repos)) => {
                self.repositories = repos;
            }
            Message::RepositoriesLoaded(Err(_)) => {
                // TODO: Show error banner
            }
            Message::TagsLoaded(repo, Ok(tags)) => {
                self.tags.insert(repo, tags);
            }
            Message::TagsLoaded(_, Err(_)) => {
                // TODO: Show error banner
            }
            Message::ManifestLoaded(_, _, Ok(_)) => {
                // TODO: Update manifest data when details view is implemented
            }
            Message::ManifestLoaded(_, _, Err(_)) => {
                // TODO: Show error banner
            }
            Message::Error(_) => {
                // TODO: Show error banner
            }
        }
    }

    /// Spawn a background worker to perform an operation.
    ///
    /// The worker should call the provided function and send the result
    /// back via the message channel.
    ///
    /// # Arguments
    ///
    /// * `f` - The function to execute in the background
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rex::tui::app::{App, Message};
    /// use rex::tui::theme::Theme;
    ///
    /// let app = App::new("localhost:5000".to_string(), Theme::dark(), false);
    /// app.spawn_worker(|| {
    ///     // Simulate fetching repositories
    ///     Message::RepositoriesLoaded(Ok(vec!["alpine".to_string()]))
    /// });
    /// ```
    pub fn spawn_worker<F>(&self, f: F)
    where
        F: FnOnce() -> Message + Send + 'static,
    {
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let msg = f();
            let _ = tx.send(msg);
        });
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
