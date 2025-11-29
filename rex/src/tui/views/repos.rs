//! Repository list view data model.
//!
//! Provides the data structure and state management for the repository list view.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::tui::progress::ProgressBar;
use crate::tui::theme::Theme;

// Re-export RepositoryItem from shared image module
pub use crate::image::RepositoryItem;

/// State for the repository list view.
#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO: Remove when integrated into main TUI loop (Task 3.3)
pub struct RepositoryListState {
    /// List of repository items
    pub items: Vec<RepositoryItem>,
    /// Currently selected item index
    pub selected: usize,
    /// Scroll offset for large lists
    pub scroll_offset: usize,
    /// Whether data is currently loading
    pub loading: bool,
    /// Filter string for searching repositories
    pub filter: String,
    /// Progress tracking for loading operations (current, total)
    pub progress: Option<(usize, usize)>,
}

#[allow(dead_code)] // TODO: Remove when integrated into main TUI loop (Task 3.3)
impl RepositoryListState {
    /// Create a new repository list state.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::views::repos::RepositoryListState;
    ///
    /// let state = RepositoryListState::new();
    /// assert_eq!(state.items.len(), 0);
    /// assert_eq!(state.selected, 0);
    /// ```
    pub fn new() -> Self {
        Self {
            items: vec![],
            selected: 0,
            scroll_offset: 0,
            loading: false,
            filter: String::new(),
            progress: None,
        }
    }

    /// Move selection to the next item.
    ///
    /// If already at the last item, selection stays at the last item.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::views::repos::{RepositoryListState, RepositoryItem};
    ///
    /// let mut state = RepositoryListState::new();
    /// state.items = vec![
    ///     RepositoryItem {
    ///         name: "alpine".to_string(),
    ///         tag_count: 5,
    ///         total_size: 1024,
    ///         last_updated: None,
    ///     },
    ///     RepositoryItem {
    ///         name: "nginx".to_string(),
    ///         tag_count: 10,
    ///         total_size: 2048,
    ///         last_updated: None,
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
    /// use rex::tui::views::repos::{RepositoryListState, RepositoryItem};
    ///
    /// let mut state = RepositoryListState::new();
    /// state.items = vec![
    ///     RepositoryItem {
    ///         name: "alpine".to_string(),
    ///         tag_count: 5,
    ///         total_size: 1024,
    ///         last_updated: None,
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
    /// use rex::tui::views::repos::{RepositoryListState, RepositoryItem};
    ///
    /// let mut state = RepositoryListState::new();
    /// state.items = vec![
    ///     RepositoryItem {
    ///         name: "alpine".to_string(),
    ///         tag_count: 5,
    ///         total_size: 1024,
    ///         last_updated: None,
    ///     },
    /// ];
    ///
    /// let item = state.selected_item();
    /// assert!(item.is_some());
    /// assert_eq!(item.unwrap().name, "alpine");
    /// ```
    pub fn selected_item(&self) -> Option<&RepositoryItem> {
        self.items.get(self.selected)
    }

    /// Get items filtered by the current filter string.
    ///
    /// Returns all items if filter is empty, otherwise returns items
    /// whose name contains the filter string.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::views::repos::{RepositoryListState, RepositoryItem};
    ///
    /// let mut state = RepositoryListState::new();
    /// state.items = vec![
    ///     RepositoryItem {
    ///         name: "alpine".to_string(),
    ///         tag_count: 5,
    ///         total_size: 1024,
    ///         last_updated: None,
    ///     },
    ///     RepositoryItem {
    ///         name: "nginx".to_string(),
    ///         tag_count: 10,
    ///         total_size: 2048,
    ///         last_updated: None,
    ///     },
    /// ];
    /// state.filter = "ng".to_string();
    ///
    /// let filtered = state.filtered_items();
    /// assert_eq!(filtered.len(), 1);
    /// assert_eq!(filtered[0].name, "nginx");
    /// ```
    pub fn filtered_items(&self) -> Vec<&RepositoryItem> {
        if self.filter.is_empty() {
            self.items.iter().collect()
        } else {
            self.items
                .iter()
                .filter(|item| item.name.contains(&self.filter))
                .collect()
        }
    }

    /// Render the repository list view.
    ///
    /// Displays a table with columns for repository name, tag count, size, and last updated.
    /// The selected row is highlighted with a selection indicator (▶).
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render to
    /// * `area` - The rectangular area to render in
    /// * `theme` - The theme to use for styling
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // If progress tracking is active and no items loaded yet, show centered progress bar
        if let Some((current, total)) = self.progress
            && self.items.is_empty()
        {
            let progress = ProgressBar::new(current, total, "Fetching image information");
            progress.render(frame, area, theme);
            return;
        }

        let items = self.filtered_items();

        // Header row
        let header = Row::new(vec![
            Cell::from("NAME"),
            Cell::from("TAGS"),
            Cell::from("SIZE"),
            Cell::from("LAST UPDATED"),
        ])
        .style(theme.title_style());

        // Data rows
        let rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    theme.selected_style()
                } else {
                    ratatui::style::Style::default()
                };

                let indicator = if i == self.selected { "▶ " } else { "  " };

                Row::new(vec![
                    Cell::from(format!("{}{}", indicator, item.name)),
                    Cell::from(item.tag_count.to_string()),
                    Cell::from(item.total_size_display.clone()),
                    Cell::from(item.last_updated.clone()),
                ])
                .style(style)
            })
            .collect();

        let widths = [
            Constraint::Percentage(40),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Percentage(30),
        ];

        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );

        frame.render_widget(table, area);
    }
}

impl Default for RepositoryListState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "repos_tests.rs"]
mod tests;
