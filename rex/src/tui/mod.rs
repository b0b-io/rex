//! TUI (Terminal User Interface) module for Rex.
//!
//! Provides an interactive terminal interface for exploring container registries.

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    widgets::{Block, Borders},
};
use std::io::{self, Stdout};
use std::time::Duration;

/// Result type for TUI operations.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Run the TUI application.
///
/// # Errors
///
/// Returns an error if terminal setup, event handling, or cleanup fails.
pub fn run() -> Result<()> {
    let mut terminal = setup_terminal()?;

    // Main loop: minimal implementation that quits on 'q'
    loop {
        terminal.draw(|f| {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Rex - Press 'q' to quit");
            f.render_widget(block, f.size());
        })?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.code == KeyCode::Char('q')
        {
            break;
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}

/// Setup terminal for TUI mode.
///
/// Enables raw mode and switches to alternate screen.
///
/// # Errors
///
/// Returns an error if terminal setup fails.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal to normal mode.
///
/// Disables raw mode and leaves alternate screen.
///
/// # Errors
///
/// Returns an error if terminal restoration fails.
fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(test)]
mod tests;
