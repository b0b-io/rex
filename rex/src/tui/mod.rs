//! TUI (Terminal User Interface) module for Rex.
//!
//! Provides an interactive terminal interface for exploring container registries.

pub mod app;
pub mod events;
pub mod shell;
pub mod theme;
pub mod views;
pub mod worker;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};
use std::time::Duration;

use events::{Event, EventHandler};
use shell::{Action, Footer, ShellLayout, TitleBar};
use theme::Theme;

/// Result type for TUI operations.
///
/// The error type is `Send + Sync` to allow passing across thread boundaries.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Run the TUI application.
///
/// # Arguments
///
/// * `ctx` - Application context with configuration
///
/// # Errors
///
/// Returns an error if terminal setup, event handling, or cleanup fails.
pub fn run(ctx: &crate::context::AppContext) -> Result<()> {
    let mut terminal = setup_terminal()?;

    // Get theme from config
    let theme = match ctx.config.tui.theme.as_str() {
        "light" => Theme::light(),
        _ => Theme::dark(), // Default to dark
    };

    // Get registry from config
    let registry = get_registry_url(&ctx.config)?;

    let title_bar = TitleBar::new().with_registry(registry.clone());
    let footer = Footer::new(vec![Action::new("?", "Help"), Action::new("q", "Quit")]);

    // Create event handler with configured vim mode
    let event_handler = EventHandler::new(ctx.config.tui.vim_mode);

    // Get configured poll interval
    let poll_interval = Duration::from_millis(ctx.config.tui.poll_interval);

    // Main loop
    loop {
        terminal.draw(|f| {
            let area = f.size();

            // Calculate shell layout (no context bar or status line for now)
            let layout = ShellLayout::calculate(area, false, false);

            // Render title bar
            title_bar.render(f, layout.title_bar, &theme);

            // TODO: Render content area (will be implemented in later tasks)

            // Render footer
            footer.render(f, layout.footer, &theme);
        })?;

        // Poll for events with configured interval
        if let Some(event) = event_handler.poll(poll_interval)? {
            match event {
                Event::Quit => break,
                Event::Resize(_, _) => {
                    // Terminal will automatically redraw on next iteration
                }
                _ => {
                    // TODO: Handle other events when views are implemented
                }
            }
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

/// Get registry URL from config.
///
/// Returns the default registry from config, or falls back to localhost:5000.
fn get_registry_url(config: &crate::config::Config) -> Result<String> {
    // Use default registry from config
    if let Some(default_name) = &config.registries.default {
        for entry in &config.registries.list {
            if entry.name == *default_name {
                return Ok(entry.url.clone());
            }
        }
    }

    // Fallback to first registry if available
    if let Some(first) = config.registries.list.first() {
        return Ok(first.url.clone());
    }

    // Fallback to localhost
    Ok("localhost:5000".to_string())
}

#[cfg(test)]
mod tests;
