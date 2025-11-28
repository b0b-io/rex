//! Banner component for displaying status messages.
//!
//! Banners appear in the context bar area and provide feedback for:
//! - Loading operations (with spinner)
//! - Warnings (dismissible)
//! - Errors (dismissible)
//! - Success messages (auto-dismiss after 5s)
//! - Info messages (dismissible)

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::time::{Duration, Instant};

use super::theme::Theme;

/// Type of banner to display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerType {
    /// Loading operation in progress (with animated spinner)
    Loading,
    /// Warning message (yellow)
    #[allow(dead_code)] // TODO: Remove when warning banners are used
    Warning,
    /// Error message (red)
    Error,
    /// Success message (green, auto-dismisses after 5s)
    #[allow(dead_code)] // TODO: Remove when success banners are used
    Success,
    /// Informational message (cyan)
    #[allow(dead_code)] // TODO: Remove when info banners are used
    Info,
}

/// A banner message displayed in the context bar.
#[derive(Debug, Clone)]
pub struct Banner {
    /// Unique identifier for the banner
    #[allow(dead_code)] // TODO: Remove when individual banner removal is implemented
    id: usize,
    /// Message to display
    message: String,
    /// Type of banner
    banner_type: BannerType,
    /// When the banner was created
    created_at: Instant,
    /// Whether the banner can be dismissed with [×]
    dismissible: bool,
}

impl Banner {
    /// Create a new banner with a unique ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the banner
    /// * `message` - Message to display
    /// * `banner_type` - Type of banner
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::banner::{Banner, BannerType};
    ///
    /// let banner = Banner::new(1, "Loading repositories...".to_string(), BannerType::Loading);
    /// ```
    pub fn new(id: usize, message: String, banner_type: BannerType) -> Self {
        let dismissible = matches!(
            banner_type,
            BannerType::Warning | BannerType::Error | BannerType::Info
        );

        Self {
            id,
            message,
            banner_type,
            created_at: Instant::now(),
            dismissible,
        }
    }

    /// Get the banner's unique ID.
    #[allow(dead_code)] // TODO: Remove when individual banner management is needed
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the banner type.
    #[allow(dead_code)] // TODO: Remove when banner type inspection is needed
    pub fn banner_type(&self) -> BannerType {
        self.banner_type
    }

    /// Check if the banner should auto-dismiss.
    ///
    /// Success banners auto-dismiss after 5 seconds.
    pub fn should_auto_dismiss(&self) -> bool {
        matches!(self.banner_type, BannerType::Success)
            && self.created_at.elapsed() > Duration::from_secs(5)
    }

    /// Get the symbol for this banner type.
    fn symbol(&self) -> &'static str {
        match self.banner_type {
            BannerType::Loading => "⟳",
            BannerType::Warning => "⚠",
            BannerType::Error => "✗",
            BannerType::Success => "✓",
            BannerType::Info => "ℹ",
        }
    }

    /// Render the banner in the given area.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render in
    /// * `theme` - The theme to use for styling
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Choose style based on banner type
        let style = match self.banner_type {
            BannerType::Loading => Style::default().fg(theme.info),
            BannerType::Warning => Style::default().fg(theme.warning),
            BannerType::Error => Style::default().fg(theme.error),
            BannerType::Success => Style::default().fg(theme.success),
            BannerType::Info => Style::default().fg(theme.info),
        };

        let mut spans = vec![
            Span::styled(format!("{} ", self.symbol()), style),
            Span::styled(&self.message, Style::default().fg(theme.foreground)),
        ];

        // Add spacing and dismiss indicator if dismissible
        if self.dismissible {
            let width = area.width as usize;
            let content_len = self.symbol().len() + 1 + self.message.len();
            let dismiss_text = "[×]";
            let dismiss_len = dismiss_text.len();

            if content_len + dismiss_len + 3 < width {
                let spacing = width - content_len - dismiss_len;
                spans.push(Span::raw(" ".repeat(spacing)));
                spans.push(Span::styled(dismiss_text, Style::default().fg(theme.muted)));
            }
        }

        // Add [cached] indicator for loading banners
        if matches!(self.banner_type, BannerType::Loading) {
            let width = area.width as usize;
            let content_len = self.symbol().len() + 1 + self.message.len();
            let cached_text = "[cached]";
            let cached_len = cached_text.len();

            if content_len + cached_len + 3 < width {
                let spacing = width - content_len - cached_len;
                spans.push(Span::raw(" ".repeat(spacing)));
                spans.push(Span::styled(
                    cached_text,
                    Style::default().fg(theme.warning),
                ));
            }
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);

        frame.render_widget(paragraph, area);
    }
}

/// Manager for multiple banners.
///
/// Handles banner lifecycle, auto-dismiss, and rendering multiple stacked banners.
#[derive(Debug)]
pub struct BannerManager {
    /// List of active banners
    banners: Vec<Banner>,
    /// Next banner ID
    next_id: usize,
}

impl BannerManager {
    /// Create a new empty banner manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::banner::BannerManager;
    ///
    /// let manager = BannerManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            banners: Vec::new(),
            next_id: 0,
        }
    }

    /// Add a banner and return its ID.
    ///
    /// # Arguments
    ///
    /// * `message` - Message to display
    /// * `banner_type` - Type of banner
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::banner::{BannerManager, BannerType};
    ///
    /// let mut manager = BannerManager::new();
    /// let id = manager.add("Loading...".to_string(), BannerType::Loading);
    /// ```
    pub fn add(&mut self, message: String, banner_type: BannerType) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let banner = Banner::new(id, message, banner_type);
        self.banners.push(banner);

        id
    }

    /// Remove a banner by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the banner to remove
    #[allow(dead_code)] // TODO: Remove when individual banner removal by ID is needed
    pub fn remove(&mut self, id: usize) {
        self.banners.retain(|b| b.id != id);
    }

    /// Remove all banners of a specific type.
    ///
    /// # Arguments
    ///
    /// * `banner_type` - Type of banners to remove
    pub fn remove_type(&mut self, banner_type: BannerType) {
        self.banners.retain(|b| b.banner_type != banner_type);
    }

    /// Remove all banners.
    #[allow(dead_code)] // TODO: Remove when clear all banners functionality is needed
    pub fn clear(&mut self) {
        self.banners.clear();
    }

    /// Process auto-dismiss for success banners.
    ///
    /// Should be called on each frame to check for expired banners.
    pub fn process_auto_dismiss(&mut self) {
        self.banners.retain(|b| !b.should_auto_dismiss());
    }

    /// Check if there are any active banners.
    pub fn has_banners(&self) -> bool {
        !self.banners.is_empty()
    }

    /// Get the number of active banners.
    #[allow(dead_code)] // TODO: Remove when banner count display is needed
    pub fn count(&self) -> usize {
        self.banners.len()
    }

    /// Render the most recent banner.
    ///
    /// Only the newest banner is shown to avoid clutter.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render in
    /// * `theme` - The theme to use for styling
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(banner) = self.banners.last() {
            banner.render(frame, area, theme);
        }
    }

    /// Dismiss the most recent dismissible banner.
    ///
    /// Called when user presses a dismiss key.
    #[allow(dead_code)] // TODO: Remove when user dismiss interaction is implemented
    pub fn dismiss_latest(&mut self) {
        // Find the most recent dismissible banner and remove it
        if let Some(pos) = self.banners.iter().rposition(|b| b.dismissible) {
            self.banners.remove(pos);
        }
    }
}

impl Default for BannerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "banner_tests.rs"]
mod tests;
