//! Application state management for Rex TUI.
//!
//! Handles the application state, view navigation, and message passing between
//! the UI thread and background workers.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use librex::Credentials;
use librex::auth::CredentialStore;

use super::Result;
use super::events::Event;
use super::theme::Theme;
use super::views::details::ImageDetailsState;
use super::views::repos::{RepositoryItem, RepositoryListState};
use super::views::tags::{TagItem, TagListState};
use super::worker;

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
    /// Repositories loaded successfully or with error (includes tag counts)
    RepositoriesLoaded(Result<Vec<RepositoryItem>>),
    /// Tags loaded for a repository
    TagsLoaded(String, Result<Vec<String>>),
    /// Manifest loaded for an image
    ManifestLoaded(String, String, Box<Result<librex::ManifestOrIndex>>),
    /// Configuration loaded for an image
    ConfigLoaded(String, String, Box<Result<librex::oci::ImageConfiguration>>),
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
    /// Cache directory for registry data
    pub cache_dir: PathBuf,
    /// Optional credentials for authentication
    pub credentials: Option<Credentials>,

    // Data (cached)
    /// List of repositories
    pub repositories: Vec<String>,
    /// Tags for each repository (keyed by repository name)
    pub tags: HashMap<String, Vec<String>>,

    // View states
    /// State for the repository list view
    pub repo_list_state: RepositoryListState,
    /// State for the tag list view
    pub tag_list_state: TagListState,
    /// State for the image details view
    pub details_state: ImageDetailsState,

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
    /// Create a new application state from the application context.
    ///
    /// Extracts registry configuration, cache directory, credentials, theme,
    /// and other settings from the provided context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The application context with configuration
    ///
    /// # Errors
    ///
    /// Returns an error if registry configuration cannot be determined or
    /// cache directory cannot be created.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rex::context::AppContext;
    /// use rex::tui::app::App;
    ///
    /// let ctx = AppContext::build(
    ///     rex::format::ColorChoice::Auto,
    ///     rex::context::VerbosityLevel::Normal
    /// );
    /// let app = App::new(&ctx).unwrap();
    /// ```
    pub fn new(ctx: &crate::context::AppContext) -> Result<Self> {
        let (tx, rx) = channel();

        // Get registry URL from config
        let registry = get_registry_url(&ctx.config)?;

        // Get cache directory for this registry
        let cache_dir = crate::config::get_registry_cache_dir(&registry)?;

        // Load credentials if available
        let creds_path = crate::config::get_credentials_path();
        let credentials = if creds_path.exists() {
            if let Ok(store) = librex::auth::FileCredentialStore::new(creds_path) {
                store.get(&registry).ok().flatten()
            } else {
                None
            }
        } else {
            None
        };

        // Get theme from config
        let theme = match ctx.config.tui.theme.as_str() {
            "light" => Theme::light(),
            _ => Theme::dark(), // Default to dark
        };

        // Get vim mode setting
        let vim_mode = ctx.config.tui.vim_mode;

        Ok(Self {
            current_view: View::RepositoryList,
            view_stack: vec![],
            should_quit: false,
            current_registry: registry,
            cache_dir,
            credentials,
            repositories: vec![],
            tags: HashMap::new(),
            repo_list_state: RepositoryListState::new(),
            tag_list_state: TagListState::default(),
            details_state: ImageDetailsState::default(),
            tx,
            rx,
            theme,
            vim_mode,
        })
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
    fn handle_repo_list_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Up => {
                self.repo_list_state.select_previous();
            }
            Event::Down => {
                self.repo_list_state.select_next();
            }
            Event::Enter => {
                // Navigate to tag list for selected repository
                if let Some(item) = self.repo_list_state.selected_item() {
                    let repo_name = item.name.clone();
                    // Initialize tag list state for this repository
                    self.tag_list_state = TagListState::new(repo_name.clone());
                    // Load tags in background
                    self.load_tags(repo_name.clone());
                    // Navigate to tag list view
                    self.push_view(View::TagList(repo_name));
                }
            }
            Event::Refresh => {
                // Reload repositories (concurrency of 8 is reasonable default)
                self.load_repositories(8);
            }
            Event::Search => {
                // TODO: Implement search mode in Phase 4
            }
            _ => {}
        }
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
    fn handle_tag_list_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Up => {
                self.tag_list_state.select_previous();
            }
            Event::Down => {
                self.tag_list_state.select_next();
            }
            Event::Enter => {
                // Navigate to image details for selected tag
                if let Some(item) = self.tag_list_state.selected_item() {
                    let repo = self.tag_list_state.repository.clone();
                    let tag = item.tag.clone();
                    // Initialize details state
                    self.details_state = ImageDetailsState::new(repo.clone(), tag.clone());
                    // Load manifest and config in background
                    self.load_manifest(repo.clone(), tag.clone());
                    // Navigate to details view
                    self.push_view(View::ImageDetails(repo, tag));
                }
            }
            Event::Refresh => {
                // Reload tags for the current repository
                let repo = self.tag_list_state.repository.clone();
                self.load_tags(repo);
            }
            Event::Search => {
                // TODO: Implement search mode in Phase 4
            }
            _ => {}
        }
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
    fn handle_details_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Up => {
                self.details_state.scroll_up();
            }
            Event::Down => {
                self.details_state.scroll_down();
            }
            Event::PageUp => {
                self.details_state.scroll_page_up();
            }
            Event::PageDown => {
                self.details_state.scroll_page_down();
            }
            Event::Home => {
                self.details_state.scroll_to_top();
            }
            Event::Refresh => {
                // Reload manifest and config
                let repo = self.details_state.repository.clone();
                let tag = self.details_state.tag.clone();
                self.load_manifest(repo, tag);
            }
            _ => {}
        }
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
            Message::RepositoriesLoaded(Ok(items)) => {
                // Update repository list with metadata
                self.repositories = items.iter().map(|item| item.name.clone()).collect();
                self.repo_list_state.items = items;
                self.repo_list_state.loading = false;
            }
            Message::RepositoriesLoaded(Err(_)) => {
                self.repo_list_state.loading = false;
                // TODO: Show error banner
            }
            Message::TagsLoaded(repo, Ok(tags)) => {
                self.tags.insert(repo.clone(), tags.clone());
                // Update tag list state if we're currently viewing this repository
                if self.tag_list_state.repository == repo {
                    // Convert tags to TagItem objects
                    self.tag_list_state.items = tags
                        .into_iter()
                        .map(|tag| TagItem {
                            tag,
                            digest: String::new(), // TODO: Fetch actual digest from manifest
                            size: 0,               // TODO: Calculate actual size
                            platforms: vec![],     // TODO: Fetch platforms from manifest
                            updated: None,         // TODO: Fetch last updated time
                        })
                        .collect();
                    self.tag_list_state.loading = false;
                }
            }
            Message::TagsLoaded(_, Err(_)) => {
                self.tag_list_state.loading = false;
                // TODO: Show error banner
            }
            Message::ManifestLoaded(repo, tag, result) => {
                // Update details state if we're currently viewing this image
                if self.details_state.repository == repo && self.details_state.tag == tag {
                    match *result {
                        Ok(manifest) => {
                            self.details_state.manifest = Some(manifest);
                            // Don't clear loading yet - wait for config to load too
                        }
                        Err(_) => {
                            self.details_state.loading = false;
                            // TODO: Show error banner
                        }
                    }
                }
            }
            Message::ConfigLoaded(repo, tag, result) => {
                // Update details state if we're currently viewing this image
                if self.details_state.repository == repo && self.details_state.tag == tag {
                    match *result {
                        Ok(config) => {
                            self.details_state.config = Some(config);
                            self.details_state.loading = false;
                        }
                        Err(_) => {
                            // Config is optional (may not exist for indexes)
                            self.details_state.loading = false;
                        }
                    }
                }
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

    /// Load repositories from the registry in a background worker.
    ///
    /// Sets the loading state and spawns a worker to fetch the repository list with
    /// metadata (tag counts). Uses parallel requests to fetch tag counts.
    /// When complete, the worker sends a `RepositoriesLoaded` message.
    ///
    /// # Arguments
    ///
    /// * `concurrency` - Maximum number of parallel connections
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use rex::tui::app::App;
    /// use rex::tui::theme::Theme;
    ///
    /// let mut app = App::new(
    ///     "localhost:5000".to_string(),
    ///     PathBuf::from("/tmp/cache"),
    ///     None,
    ///     Theme::dark(),
    ///     false
    /// );
    /// app.load_repositories(8);
    /// assert!(app.repo_list_state.loading);
    /// ```
    pub fn load_repositories(&mut self, concurrency: usize) {
        self.repo_list_state.loading = true;
        let registry_url = self.current_registry.clone();
        let cache_dir = self.cache_dir.clone();
        let credentials = self.credentials.clone();
        let tx = self.tx.clone();

        // Spawn worker thread to fetch repositories with metadata
        std::thread::spawn(move || {
            worker::fetch_repositories(registry_url, &cache_dir, credentials, tx, concurrency);
        });
    }

    /// Load tags for a specific repository in a background worker.
    ///
    /// Sets the loading state and spawns a worker to fetch the tag list.
    /// When complete, the worker sends a `TagsLoaded` message.
    ///
    /// # Arguments
    ///
    /// * `repository` - The name of the repository to load tags for
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use rex::tui::app::App;
    /// use rex::tui::theme::Theme;
    ///
    /// let mut app = App::new(
    ///     "localhost:5000".to_string(),
    ///     PathBuf::from("/tmp/cache"),
    ///     None,
    ///     Theme::dark(),
    ///     false
    /// );
    /// app.load_tags("alpine".to_string());
    /// assert!(app.tag_list_state.loading);
    /// ```
    pub fn load_tags(&mut self, repository: String) {
        self.tag_list_state.loading = true;
        let registry_url = self.current_registry.clone();
        let cache_dir = self.cache_dir.clone();
        let credentials = self.credentials.clone();
        let tx = self.tx.clone();

        // Spawn worker thread to fetch tags
        std::thread::spawn(move || {
            worker::fetch_tags(registry_url, repository, &cache_dir, credentials, tx);
        });
    }

    /// Load manifest and config for a specific image in a background worker.
    ///
    /// Sets the loading state and spawns a worker to fetch the manifest and config.
    /// When complete, the worker sends `ManifestLoaded` and `ConfigLoaded` messages.
    ///
    /// # Arguments
    ///
    /// * `repository` - The name of the repository
    /// * `tag` - The tag name
    pub fn load_manifest(&mut self, repository: String, tag: String) {
        self.details_state.loading = true;
        let registry_url = self.current_registry.clone();
        let cache_dir = self.cache_dir.clone();
        let credentials = self.credentials.clone();
        let tx = self.tx.clone();

        // Spawn worker thread to fetch manifest and config
        std::thread::spawn(move || {
            worker::fetch_manifest_and_config(
                registry_url,
                repository,
                tag,
                &cache_dir,
                credentials,
                tx,
            );
        });
    }
}

/// Get registry URL from config.
///
/// Returns the default registry from config, or falls back to localhost:5000.
fn get_registry_url(config: &crate::config::Config) -> Result<String> {
    // Use default registry from config
    if let Some(default_name) = &config.registries.default {
        for entry in &config.registries.list {
            if entry.name == *default_name {
                return Ok(entry.url.clone());
            }
        }
    }

    // Fallback to first registry if available
    if let Some(first) = config.registries.list.first() {
        return Ok(first.url.clone());
    }

    // Fallback to localhost
    Ok("localhost:5000".to_string())
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
