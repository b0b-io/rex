//! Progress bar widget for displaying operation progress.
//!
//! Provides visual feedback for long-running operations with known total counts.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::theme::Theme;

/// Progress bar widget for displaying operation progress.
///
/// Shows a visual progress bar with current/total counts and percentage.
///
/// # Examples
///
/// ```
/// use rex::tui::progress::ProgressBar;
///
/// let progress = ProgressBar::new(45, 115, "Fetching repositories");
/// ```
#[derive(Debug, Clone)]
pub struct ProgressBar {
    /// Current progress count
    current: usize,
    /// Total count
    total: usize,
    /// Message to display
    message: String,
}

impl ProgressBar {
    /// Create a new progress bar.
    ///
    /// # Arguments
    ///
    /// * `current` - Current progress count
    /// * `total` - Total count
    /// * `message` - Message to display above the progress bar
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::progress::ProgressBar;
    ///
    /// let progress = ProgressBar::new(45, 115, "Fetching repositories");
    /// ```
    pub fn new<S: Into<String>>(current: usize, total: usize, message: S) -> Self {
        Self {
            current,
            total,
            message: message.into(),
        }
    }

    /// Get the current progress count.
    #[allow(dead_code)] // TODO: Remove when used
    pub fn current(&self) -> usize {
        self.current
    }

    /// Get the total count.
    #[allow(dead_code)] // TODO: Remove when used
    pub fn total(&self) -> usize {
        self.total
    }

    /// Get the percentage complete (0-100).
    pub fn percentage(&self) -> u8 {
        if self.total == 0 {
            return 0;
        }
        ((self.current as f64 / self.total as f64) * 100.0).min(100.0) as u8
    }

    /// Check if the progress bar is complete.
    #[allow(dead_code)] // TODO: Remove when used
    pub fn is_complete(&self) -> bool {
        self.current >= self.total
    }

    /// Render the progress bar in a centered area.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render in (will be centered)
    /// * `theme` - The theme to use for styling
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Calculate centered area for progress display
        let center_x = area.width / 2;
        let center_y = area.height / 2;

        // Progress bar width (50 characters total)
        let bar_width = 50u16.min(area.width.saturating_sub(10));
        let filled = if self.total > 0 {
            ((bar_width as f64) * (self.current as f64) / (self.total as f64)) as u16
        } else {
            0
        };

        // Build progress bar visualization
        let filled_str = "█".repeat(filled as usize);
        let empty_str = "░".repeat((bar_width - filled) as usize);
        let percentage = self.percentage();

        // Create spans for the message and progress bar
        let message_line = Line::from(vec![Span::styled(
            &self.message,
            Style::default().fg(theme.foreground),
        )]);

        let progress_line = Line::from(vec![
            Span::raw("["),
            Span::styled(filled_str, Style::default().fg(theme.info)),
            Span::styled(empty_str, Style::default().fg(theme.muted)),
            Span::raw("]"),
            Span::raw(format!(
                " {}/{} ({}%)",
                self.current, self.total, percentage
            )),
        ]);

        // Render message
        if center_y >= 2 {
            let message_area = Rect {
                x: center_x.saturating_sub(self.message.len() as u16 / 2),
                y: center_y - 2,
                width: self.message.len() as u16,
                height: 1,
            };

            if message_area.x + message_area.width <= area.x + area.width {
                let paragraph = Paragraph::new(message_line).alignment(Alignment::Left);
                frame.render_widget(paragraph, message_area);
            }
        }

        // Render progress bar
        let bar_area = Rect {
            x: center_x.saturating_sub(bar_width / 2 + 5),
            y: center_y,
            width: bar_width + 20, // Bar + count text
            height: 1,
        };

        if bar_area.x + bar_area.width <= area.x + area.width {
            let paragraph = Paragraph::new(progress_line).alignment(Alignment::Left);
            frame.render_widget(paragraph, bar_area);
        }
    }

    /// Render a compact progress bar (for banner area).
    ///
    /// Renders a smaller version suitable for the context bar.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render in (single line)
    /// * `theme` - The theme to use for styling
    #[allow(dead_code)] // TODO: Remove when compact progress bars are used in banners
    pub fn render_compact(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let bar_width = 20u16.min(area.width.saturating_sub(30));
        let filled = if self.total > 0 {
            ((bar_width as f64) * (self.current as f64) / (self.total as f64)) as u16
        } else {
            0
        };

        let filled_str = "█".repeat(filled as usize);
        let empty_str = "░".repeat((bar_width - filled) as usize);
        let percentage = self.percentage();

        let line = Line::from(vec![
            Span::styled("⟳ ", Style::default().fg(theme.info)),
            Span::styled(&self.message, Style::default().fg(theme.foreground)),
            Span::raw(" ["),
            Span::styled(filled_str, Style::default().fg(theme.info)),
            Span::styled(empty_str, Style::default().fg(theme.muted)),
            Span::raw("] "),
            Span::styled(
                format!("{}/{} ({}%)", self.current, self.total, percentage),
                Style::default().fg(theme.foreground),
            ),
        ]);

        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
#[path = "progress_tests.rs"]
mod tests;
