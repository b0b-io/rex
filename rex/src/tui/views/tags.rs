//! Tag list view data model.
//!
//! Provides the data structure and state management for the tag list view.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::tui::theme::Theme;

/// A tag item in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // TODO: Remove when integrated into main TUI loop
pub struct TagItem {
    /// Tag name
    pub tag: String,
    /// Content digest
    pub digest: String,
    /// Total size in bytes
    pub size: u64,
    /// List of platforms (e.g., "linux/amd64", "linux/arm64")
    pub platforms: Vec<String>,
    /// Last updated timestamp (optional)
    pub updated: Option<String>,
}

/// State for the tag list view.
#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO: Remove when integrated into main TUI loop
pub struct TagListState {
    /// Repository name this tag list belongs to
    pub repository: String,
    /// List of tag items
    pub items: Vec<TagItem>,
    /// Currently selected item index
    pub selected: usize,
    /// Scroll offset for large lists
    pub scroll_offset: usize,
    /// Whether data is currently loading
    pub loading: bool,
}

#[allow(dead_code)] // TODO: Remove when integrated into main TUI loop
impl TagListState {
    /// Create a new tag list state for a repository.
    ///
    /// # Arguments
    ///
    /// * `repository` - The name of the repository
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::views::tags::TagListState;
    ///
    /// let state = TagListState::new("alpine".to_string());
    /// assert_eq!(state.repository, "alpine");
    /// assert_eq!(state.items.len(), 0);
    /// ```
    pub fn new(repository: String) -> Self {
        Self {
            repository,
            items: vec![],
            selected: 0,
            scroll_offset: 0,
            loading: false,
        }
    }

    /// Move selection to the next item.
    ///
    /// If already at the last item, selection stays at the last item.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::views::tags::{TagListState, TagItem};
    ///
    /// let mut state = TagListState::new("alpine".to_string());
    /// state.items = vec![
    ///     TagItem {
    ///         tag: "latest".to_string(),
    ///         digest: "sha256:abc123".to_string(),
    ///         size: 1024,
    ///         platforms: vec!["linux/amd64".to_string()],
    ///         updated: None,
    ///     },
    ///     TagItem {
    ///         tag: "3.19".to_string(),
    ///         digest: "sha256:def456".to_string(),
    ///         size: 2048,
    ///         platforms: vec!["linux/amd64".to_string()],
    ///         updated: None,
    ///     },
    /// ];
    ///
    /// assert_eq!(state.selected, 0);
    /// state.select_next();
    /// assert_eq!(state.selected, 1);
    /// ```
    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Move selection to the previous item.
    ///
    /// If already at the first item, selection stays at the first item.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::views::tags::{TagListState, TagItem};
    ///
    /// let mut state = TagListState::new("alpine".to_string());
    /// state.items = vec![
    ///     TagItem {
    ///         tag: "latest".to_string(),
    ///         digest: "sha256:abc123".to_string(),
    ///         size: 1024,
    ///         platforms: vec!["linux/amd64".to_string()],
    ///         updated: None,
    ///     },
    /// ];
    /// state.selected = 0;
    ///
    /// state.select_previous();
    /// assert_eq!(state.selected, 0); // Stays at 0
    /// ```
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Get the currently selected item.
    ///
    /// Returns `None` if the list is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::views::tags::{TagListState, TagItem};
    ///
    /// let mut state = TagListState::new("alpine".to_string());
    /// state.items = vec![
    ///     TagItem {
    ///         tag: "latest".to_string(),
    ///         digest: "sha256:abc123".to_string(),
    ///         size: 1024,
    ///         platforms: vec!["linux/amd64".to_string()],
    ///         updated: None,
    ///     },
    /// ];
    ///
    /// let item = state.selected_item();
    /// assert!(item.is_some());
    /// assert_eq!(item.unwrap().tag, "latest");
    /// ```
    pub fn selected_item(&self) -> Option<&TagItem> {
        self.items.get(self.selected)
    }

    /// Render the tag list view.
    ///
    /// Displays a table with columns for tag name, digest, size, platforms, and last updated.
    /// The selected row is highlighted with a selection indicator (▶).
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render to
    /// * `area` - The rectangular area to render in
    /// * `theme` - The theme to use for styling
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Header row
        let header = Row::new(vec![
            Cell::from("TAG"),
            Cell::from("DIGEST"),
            Cell::from("SIZE"),
            Cell::from("PLATFORMS"),
        ])
        .style(theme.title_style());

        // Data rows
        let rows: Vec<Row> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    theme.selected_style()
                } else {
                    ratatui::style::Style::default()
                };

                let indicator = if i == self.selected { "▶ " } else { "  " };

                // Truncate digest to first 12 characters for display
                let digest_display = if item.digest.is_empty() {
                    "-".to_string()
                } else if item.digest.len() > 12 {
                    format!("{}...", &item.digest[..12])
                } else {
                    item.digest.clone()
                };

                // Format size
                let size_display = if item.size == 0 {
                    "-".to_string()
                } else {
                    librex::format::format_size(item.size)
                };

                // Format platforms
                let platforms_display = if item.platforms.is_empty() {
                    "-".to_string()
                } else {
                    item.platforms.join(", ")
                };

                Row::new(vec![
                    Cell::from(format!("{}{}", indicator, item.tag)),
                    Cell::from(digest_display),
                    Cell::from(size_display),
                    Cell::from(platforms_display),
                ])
                .style(style)
            })
            .collect();

        let widths = [
            Constraint::Percentage(25),
            Constraint::Length(15),
            Constraint::Length(12),
            Constraint::Percentage(40),
        ];

        let title = format!(" Tags for {} ", self.repository);
        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title(title),
        );

        frame.render_widget(table, area);
    }
}

impl Default for TagListState {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
#[path = "tags_tests.rs"]
mod tests;
