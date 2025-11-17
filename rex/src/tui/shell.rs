//! Shell components for Rex TUI.
//!
//! Provides the common layout structure and components used across all views.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::theme::Theme;

/// Layout structure for the TUI shell.
///
/// Defines the screen areas for each component of the shell.
#[derive(Debug, Clone)]
pub struct ShellLayout {
    /// Area for the title bar (always visible)
    pub title_bar: Rect,
    /// Area for the context bar (conditional - breadcrumbs/banners)
    #[allow(dead_code)] // TODO: Remove when context bar is implemented
    pub context_bar: Option<Rect>,
    /// Area for the main content (always visible)
    #[allow(dead_code)] // TODO: Remove when content area is used
    pub content: Rect,
    /// Area for the status line (conditional - counts/hints)
    #[allow(dead_code)] // TODO: Remove when status line is implemented
    pub status_line: Option<Rect>,
    /// Area for the footer (always visible)
    pub footer: Rect,
}

impl ShellLayout {
    /// Calculate shell layout based on terminal size and component visibility.
    ///
    /// # Arguments
    ///
    /// * `area` - The terminal screen area
    /// * `has_context` - Whether to show the context bar
    /// * `has_status` - Whether to show the status line
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::shell::ShellLayout;
    /// use ratatui::layout::Rect;
    ///
    /// let area = Rect::new(0, 0, 80, 24);
    /// let layout = ShellLayout::calculate(area, true, true);
    /// ```
    pub fn calculate(area: Rect, has_context: bool, has_status: bool) -> Self {
        let mut constraints = vec![
            Constraint::Length(3), // Title bar (border + line + border)
        ];

        if has_context {
            constraints.push(Constraint::Length(1)); // Context bar
        }

        constraints.push(Constraint::Min(0)); // Content (fills remaining)

        if has_status {
            constraints.push(Constraint::Length(1)); // Status line
        }

        constraints.push(Constraint::Length(3)); // Footer (border + line + border)

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let mut idx = 0;

        // Title bar is always first
        let title_bar = chunks[idx];
        idx += 1;

        // Context bar (if present)
        let context_bar = if has_context {
            let rect = chunks[idx];
            idx += 1;
            Some(rect)
        } else {
            None
        };

        // Content area
        let content = chunks[idx];
        idx += 1;

        // Status line (if present)
        let status_line = if has_status {
            let rect = chunks[idx];
            idx += 1;
            Some(rect)
        } else {
            None
        };

        // Footer is always last
        let footer = chunks[idx];

        Self {
            title_bar,
            context_bar,
            content,
            status_line,
            footer,
        }
    }
}

/// Title bar component showing app name and current registry.
#[derive(Debug, Clone)]
pub struct TitleBar {
    /// Application name
    pub app_name: String,
    /// Current registry name
    pub registry_name: Option<String>,
}

impl TitleBar {
    /// Create a new title bar with default app name.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::shell::TitleBar;
    ///
    /// let title_bar = TitleBar::new();
    /// ```
    pub fn new() -> Self {
        Self {
            app_name: "Rex".to_string(),
            registry_name: None,
        }
    }

    /// Set the current registry name.
    ///
    /// # Arguments
    ///
    /// * `name` - The registry name to display
    #[allow(dead_code)] // TODO: Remove when used for dynamic registry switching
    pub fn set_registry(&mut self, name: String) {
        self.registry_name = Some(name);
    }

    /// Create a title bar with a registry name (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `name` - The registry name to display
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::shell::TitleBar;
    ///
    /// let title_bar = TitleBar::new().with_registry("localhost:5000".to_string());
    /// ```
    pub fn with_registry(mut self, name: String) -> Self {
        self.registry_name = Some(name);
        self
    }

    /// Format the title bar text with proper spacing.
    ///
    /// # Arguments
    ///
    /// * `width` - Available width for the title bar
    ///
    /// # Returns
    ///
    /// Formatted string with app name on left and registry info on right
    pub fn format_text(&self, width: u16) -> String {
        let width = width as usize;

        if let Some(ref registry) = self.registry_name {
            let left = &self.app_name;
            let right = format!("Registry: {}   [r]", registry);

            let total_len = left.len() + right.len();

            if total_len < width {
                // Normal case: add spacing between left and right
                let spacing = width - total_len;
                format!("{}{}{}", left, " ".repeat(spacing), right)
            } else if total_len == width {
                // Exact fit
                format!("{}{}", left, right)
            } else {
                // Too narrow: truncate or show only essential parts
                if width > right.len() {
                    // Show truncated app name + registry
                    let available = width - right.len();
                    format!("{}{}", &left[..available.min(left.len())], right)
                } else if width > 10 {
                    // Show just registry (truncated if needed)
                    right[..width.min(right.len())].to_string()
                } else {
                    // Very narrow: show what we can
                    left[..width.min(left.len())].to_string()
                }
            }
        } else {
            // No registry: just app name
            if width >= self.app_name.len() {
                self.app_name.clone()
            } else {
                self.app_name[..width.min(self.app_name.len())].to_string()
            }
        }
    }

    /// Render the title bar to the given frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render within
    /// * `theme` - The theme to use for styling
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(theme.border_style());

        let inner = block.inner(area);

        let text = self.format_text(inner.width);
        let line = Line::from(Span::styled(text, theme.title_style()));
        let paragraph = Paragraph::new(line);

        frame.render_widget(block, area);
        frame.render_widget(paragraph, inner);
    }
}

impl Default for TitleBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Action representing a key binding in the footer.
#[derive(Debug, Clone)]
pub struct Action {
    /// The key to press
    pub key: String,
    /// Description of what the key does
    pub description: String,
    /// Whether the action is enabled
    pub enabled: bool,
}

impl Action {
    /// Create a new enabled action.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to press
    /// * `description` - What the key does
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::shell::Action;
    ///
    /// let action = Action::new("q", "Quit");
    /// ```
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            enabled: true,
        }
    }

    /// Mark this action as disabled (builder pattern).
    #[allow(dead_code)] // TODO: Remove when dynamic action enabling/disabling is needed
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Mark this action as enabled (builder pattern).
    #[allow(dead_code)] // TODO: Remove when dynamic action enabling/disabling is needed
    pub fn enabled(mut self) -> Self {
        self.enabled = true;
        self
    }
}

impl From<(&str, &str)> for Action {
    fn from((key, description): (&str, &str)) -> Self {
        Self::new(key, description)
    }
}

/// Footer component showing available key bindings.
#[derive(Debug, Clone)]
pub struct Footer {
    /// List of actions
    pub actions: Vec<Action>,
}

impl Footer {
    /// Create a new footer with the given actions.
    ///
    /// # Arguments
    ///
    /// * `actions` - The list of actions to display
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::shell::{Footer, Action};
    ///
    /// let footer = Footer::new(vec![
    ///     Action::new("q", "Quit"),
    ///     Action::new("?", "Help"),
    /// ]);
    /// ```
    pub fn new(actions: Vec<Action>) -> Self {
        Self { actions }
    }

    /// Format the footer text with all actions.
    ///
    /// # Returns
    ///
    /// A formatted string with all actions separated by spacing
    #[allow(dead_code)] // TODO: Remove when text-only rendering is needed
    pub fn format_text(&self) -> String {
        self.actions
            .iter()
            .map(|action| format!("[{}] {}", action.key, action.description))
            .collect::<Vec<_>>()
            .join("  ")
    }

    /// Render the footer to the given frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render within
    /// * `theme` - The theme to use for styling
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(theme.border_style());

        let inner = block.inner(area);

        let mut spans = vec![];
        for (i, action) in self.actions.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("  "));
            }

            let key_style = if action.enabled {
                theme.info_style()
            } else {
                theme.muted_style()
            };

            let desc_style = if action.enabled {
                Style::default().fg(theme.foreground)
            } else {
                theme.muted_style()
            };

            spans.push(Span::styled(format!("[{}]", action.key), key_style));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(&action.description, desc_style));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);

        frame.render_widget(block, area);
        frame.render_widget(paragraph, inner);
    }
}

impl From<&[(&str, &str)]> for Footer {
    fn from(actions: &[(&str, &str)]) -> Self {
        Self::new(actions.iter().map(|&(k, d)| Action::from((k, d))).collect())
    }
}

#[cfg(test)]
#[path = "shell_tests.rs"]
mod shell_layout_tests;

#[cfg(test)]
#[path = "title_bar_tests.rs"]
mod title_bar_tests;

#[cfg(test)]
#[path = "footer_tests.rs"]
mod footer_tests;
