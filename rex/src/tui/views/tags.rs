//! Tag list view data model.
//!
//! Provides the data structure and state management for the tag list view.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::tui::theme::Theme;

// Re-export TagInfo from shared image module as TagItem for TUI
pub use crate::image::TagInfo as TagItem;

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
        // Header row - matches CLI output
        let header = Row::new(vec![
            Cell::from("TAG"),
            Cell::from("DIGEST"),
            Cell::from("SIZE"),
            Cell::from("CREATED"),
            Cell::from("PLATFORM"),
        ])
        .style(theme.title_style());

        // Data rows - use pre-formatted strings from TagInfo
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

                // All fields are already formatted in TagInfo
                Row::new(vec![
                    Cell::from(format!("{}{}", indicator, item.tag)),
                    Cell::from(item.digest.clone()),
                    Cell::from(item.size.clone()),
                    Cell::from(item.created.clone()),
                    Cell::from(item.platforms.clone()),
                ])
                .style(style)
            })
            .collect();

        let widths = [
            Constraint::Percentage(25), // TAG
            Constraint::Length(13),     // DIGEST (12 chars + space)
            Constraint::Length(11),     // SIZE (e.g., "41.23 MiB")
            Constraint::Length(13),     // CREATED (e.g., "2 days ago")
            Constraint::Percentage(30), // PLATFORM
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
