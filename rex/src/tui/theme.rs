//! Theme system for Rex TUI.
//!
//! Provides color schemes and styling helpers for the terminal UI.

use ratatui::style::{Color, Modifier, Style};

/// Theme defining colors and styles for the TUI.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Background color
    #[allow(dead_code)] // TODO: Remove when used for background styling
    pub background: Color,
    /// Foreground (text) color
    pub foreground: Color,
    /// Border color (normal state)
    pub border: Color,
    /// Border color (focused state)
    #[allow(dead_code)] // TODO: Remove when focused borders are implemented
    pub border_focused: Color,
    /// Selected item background color
    #[allow(dead_code)] // TODO: Remove when selection is implemented
    pub selected_bg: Color,
    /// Selected item foreground color
    #[allow(dead_code)] // TODO: Remove when selection is implemented
    pub selected_fg: Color,
    /// Success/positive action color
    #[allow(dead_code)] // TODO: Remove when success messages are implemented
    pub success: Color,
    /// Warning/caution color
    #[allow(dead_code)] // TODO: Remove when warning messages are implemented
    pub warning: Color,
    /// Error/danger color
    #[allow(dead_code)] // TODO: Remove when error messages are implemented
    pub error: Color,
    /// Info/neutral color
    pub info: Color,
    /// Muted/disabled color
    pub muted: Color,
}

impl Theme {
    /// Create a dark theme (Catppuccin Mocha inspired).
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::theme::Theme;
    ///
    /// let theme = Theme::dark();
    /// ```
    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(30, 30, 46),        // #1e1e2e
            foreground: Color::Rgb(205, 214, 244),     // #cdd6f4
            border: Color::Rgb(69, 71, 90),            // #45475a
            border_focused: Color::Rgb(137, 180, 250), // #89b4fa (blue)
            selected_bg: Color::Rgb(49, 50, 68),       // #313244
            selected_fg: Color::Rgb(137, 180, 250),    // #89b4fa (blue)
            success: Color::Rgb(166, 227, 161),        // #a6e3a1 (green)
            warning: Color::Rgb(249, 226, 175),        // #f9e2af (yellow)
            error: Color::Rgb(243, 139, 168),          // #f38ba8 (red)
            info: Color::Rgb(137, 220, 235),           // #89dceb (cyan)
            muted: Color::Rgb(108, 112, 134),          // #6c7086 (gray)
        }
    }

    /// Create a light theme (Catppuccin Latte inspired).
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::theme::Theme;
    ///
    /// let theme = Theme::light();
    /// ```
    #[allow(dead_code)] // TODO: Remove when light theme option is added
    pub fn light() -> Self {
        Self {
            background: Color::Rgb(239, 241, 245),    // #eff1f5
            foreground: Color::Rgb(76, 79, 105),      // #4c4f69
            border: Color::Rgb(172, 176, 190),        // #acb0be
            border_focused: Color::Rgb(30, 102, 245), // #1e66f5 (blue)
            selected_bg: Color::Rgb(220, 224, 232),   // #dce0e8
            selected_fg: Color::Rgb(30, 102, 245),    // #1e66f5 (blue)
            success: Color::Rgb(64, 160, 43),         // #40a02b (green)
            warning: Color::Rgb(223, 142, 29),        // #df8e1d (orange)
            error: Color::Rgb(210, 15, 57),           // #d20f39 (red)
            info: Color::Rgb(4, 165, 229),            // #04a5e5 (cyan)
            muted: Color::Rgb(156, 160, 176),         // #9ca0b0 (gray)
        }
    }

    /// Style for titles (bold, foreground color).
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.foreground)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for normal borders.
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Style for focused borders.
    #[allow(dead_code)] // TODO: Remove when focused borders are implemented
    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    /// Style for selected items.
    #[allow(dead_code)] // TODO: Remove when selection is implemented
    pub fn selected_style(&self) -> Style {
        Style::default().bg(self.selected_bg).fg(self.selected_fg)
    }

    /// Style for success messages.
    #[allow(dead_code)] // TODO: Remove when success messages are implemented
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Style for warning messages.
    #[allow(dead_code)] // TODO: Remove when warning messages are implemented
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Style for error messages.
    #[allow(dead_code)] // TODO: Remove when error messages are implemented
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Style for info messages.
    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info)
    }

    /// Style for muted/disabled text.
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;
