//! TUI (Terminal User Interface) module for Rex.
//!
//! Provides an interactive terminal interface for exploring container registries.

pub mod shell;
pub mod theme;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};
use std::time::Duration;

use shell::{ShellLayout, TitleBar};
use theme::Theme;

/// Result type for TUI operations.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Run the TUI application.
///
/// # Errors
///
/// Returns an error if terminal setup, event handling, or cleanup fails.
pub fn run() -> Result<()> {
    let mut terminal = setup_terminal()?;

    // Create theme and components
    let theme = Theme::dark();
    let title_bar = TitleBar::new().with_registry("localhost:5000".to_string());

    // Main loop
    loop {
        terminal.draw(|f| {
            let area = f.size();

            // Calculate shell layout (no context bar or status line for now)
            let layout = ShellLayout::calculate(area, false, false);

            // Render title bar
            title_bar.render(f, layout.title_bar, &theme);

            // TODO: Render content area (will be implemented in later tasks)
            // TODO: Render footer (Task 1.6)
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
