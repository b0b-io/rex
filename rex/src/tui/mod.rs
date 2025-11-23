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

use app::App;
use events::EventHandler;
use shell::{Action, Footer, ShellLayout, TitleBar};

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

    // Create app state from context (extracts registry, cache, credentials, theme, etc.)
    let mut app = App::new(ctx)?;

    let title_bar = TitleBar::new().with_registry(app.current_registry.clone());
    let footer = Footer::new(vec![
        Action::new("↑↓", "Navigate"),
        Action::new("Enter", "Select"),
        Action::new("R", "Refresh"),
        Action::new("q", "Quit"),
    ]);

    // Load repositories on startup with configured concurrency
    app.load_repositories(ctx.config.concurrency);

    // Create event handler with configured vim mode
    let event_handler = EventHandler::new(ctx.config.tui.vim_mode);

    // Get configured poll interval
    let poll_interval = Duration::from_millis(ctx.config.tui.poll_interval);

    // Main loop
    loop {
        // Process any pending messages from workers
        app.process_messages();

        terminal.draw(|f| {
            let area = f.size();

            // Calculate shell layout (no context bar or status line for now)
            let layout = ShellLayout::calculate(area, false, false);

            // Render title bar
            title_bar.render(f, layout.title_bar, &app.theme);

            // Render content based on current view
            match &app.current_view {
                app::View::RepositoryList => {
                    app.repo_list_state.render(f, layout.content, &app.theme);
                }
                app::View::TagList(_) => {
                    app.tag_list_state.render(f, layout.content, &app.theme);
                }
                app::View::ImageDetails(_, _) => {
                    app.details_state.render(f, layout.content, &app.theme);
                }
                _ => {
                    // TODO: Implement other views (RegistrySelector, HelpPanel)
                }
            }

            // Render footer
            footer.render(f, layout.footer, &app.theme);
        })?;

        // Check if app wants to quit
        if app.should_quit {
            break;
        }

        // Poll for events with configured interval
        if let Some(event) = event_handler.poll(poll_interval)? {
            // Handle event through app
            app.handle_event(event)?;
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
