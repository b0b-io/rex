//! Event handling for Rex TUI.
//!
//! Maps crossterm keyboard events to application-level events.

use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use super::Result;

/// Application-level events for the TUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    // Navigation
    /// Move up (↑ or k in vim mode)
    Up,
    /// Move down (↓ or j in vim mode)
    Down,
    /// Move left (← or h in vim mode)
    Left,
    /// Move right (→ or l in vim mode)
    Right,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Jump to top (Home or gg in vim mode)
    Home,
    /// Jump to bottom (End or G in vim mode)
    End,
    /// Select/confirm (Enter)
    Enter,
    /// Switch focus (Tab)
    Tab,

    // Actions
    /// Open search (/)
    Search,
    /// Refresh data (R - uppercase)
    Refresh,
    /// Delete item (d)
    Delete,
    /// Copy reference (y)
    Copy,
    /// Toggle help (?)
    Help,
    /// Open registry selector (r)
    RegistrySelector,
    /// Inspect/view details (i)
    Inspect,

    // Special
    /// Quit application (q)
    Quit,
    /// Go back/cancel (Esc, or h in some contexts)
    Back,
    /// Any other character (for text input)
    Char(char),

    // System
    /// Terminal resized
    Resize(u16, u16),
}

/// Event handler that maps keyboard input to application events.
pub struct EventHandler {
    /// Whether vim mode is enabled (hjkl navigation)
    vim_mode: bool,
}

impl EventHandler {
    /// Create a new event handler.
    ///
    /// # Arguments
    ///
    /// * `vim_mode` - Whether to enable vim-style navigation keys (hjkl)
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::events::EventHandler;
    ///
    /// let handler = EventHandler::new(true);
    /// ```
    pub fn new(vim_mode: bool) -> Self {
        Self { vim_mode }
    }

    /// Poll for events with a timeout.
    ///
    /// Returns `None` if no event is available within the timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - How long to wait for an event
    ///
    /// # Errors
    ///
    /// Returns an error if event polling fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rex::tui::events::EventHandler;
    /// use std::time::Duration;
    ///
    /// let handler = EventHandler::new(false);
    /// if let Ok(Some(event)) = handler.poll(Duration::from_millis(100)) {
    ///     // Handle event
    /// }
    /// ```
    pub fn poll(&self, timeout: Duration) -> Result<Option<Event>> {
        if !event::poll(timeout)? {
            return Ok(None);
        }

        match event::read()? {
            CrosstermEvent::Key(key) => Ok(Some(self.handle_key(key))),
            CrosstermEvent::Resize(w, h) => Ok(Some(Event::Resize(w, h))),
            _ => Ok(None),
        }
    }

    /// Map a crossterm key event to an application event.
    ///
    /// # Arguments
    ///
    /// * `key` - The crossterm key event
    ///
    /// # Returns
    ///
    /// The corresponding application event
    fn handle_key(&self, key: KeyEvent) -> Event {
        // Handle Ctrl+C as quit
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Event::Quit;
        }

        match key.code {
            // Action keys (explicit character matches)
            KeyCode::Char('q') => Event::Quit,
            KeyCode::Char('/') => Event::Search,
            KeyCode::Char('r') => Event::RegistrySelector,
            KeyCode::Char('R') => Event::Refresh,
            KeyCode::Char('d') => Event::Delete,
            KeyCode::Char('y') => Event::Copy,
            KeyCode::Char('?') => Event::Help,
            KeyCode::Char('i') => Event::Inspect,

            // Vim mode navigation (only if enabled)
            KeyCode::Char('k') if self.vim_mode => Event::Up,
            KeyCode::Char('j') if self.vim_mode => Event::Down,
            KeyCode::Char('h') if self.vim_mode => Event::Left,
            KeyCode::Char('l') if self.vim_mode => Event::Right,

            // Standard arrow keys (always work)
            KeyCode::Up => Event::Up,
            KeyCode::Down => Event::Down,
            KeyCode::Left => Event::Left,
            KeyCode::Right => Event::Right,

            // Special navigation keys
            KeyCode::Enter => Event::Enter,
            KeyCode::Esc => Event::Back,
            KeyCode::Tab => Event::Tab,
            KeyCode::PageUp => Event::PageUp,
            KeyCode::PageDown => Event::PageDown,
            KeyCode::Home => Event::Home,
            KeyCode::End => Event::End,

            // Any other character
            KeyCode::Char(c) => Event::Char(c),

            // Ignore all other keys
            _ => Event::Char('\0'),
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
#[path = "events_tests.rs"]
mod tests;
