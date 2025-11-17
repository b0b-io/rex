# Rex TUI - Implementation Plan

## Overview

This document outlines the implementation tasks for Rex TUI in dependency order. Each task is designed to be independently testable and follows the TDD approach outlined in `dev.md`.

---

## Phase 1: Foundation (Weeks 1-2)

### Task 1.1: Project Structure & Dependencies
**Dependencies:** None
**Effort:** ~1 hour
**Deliverable:** Cargo.toml updated, module structure created

```rust
// rex/src/tui/mod.rs structure
rex/src/tui/
├── mod.rs              // Public API
├── app.rs              // Application state
├── shell.rs            // Shell components
├── theme.rs            // Theme and colors
├── events.rs           // Event handling
├── worker.rs           // Background workers
└── views/
    ├── mod.rs
    ├── repos.rs
    ├── tags.rs
    └── details.rs
```

**Dependencies to add:**
```toml
[dependencies]
ratatui = "0.25"
crossterm = "0.27"
```

**Acceptance Criteria:**
- [x] Module structure created
- [x] Dependencies compile
- [x] Basic `mod.rs` exports defined
- [x] Runs `cargo check` successfully

---

### Task 1.2: Terminal Initialization & Cleanup
**Dependencies:** 1.1
**Effort:** ~2 hours
**Deliverable:** Terminal setup/restore working correctly

```rust
// rex/src/tui/mod.rs
pub fn run() -> Result<()> {
    let mut terminal = setup_terminal()?;

    // Main loop placeholder
    loop {
        terminal.draw(|f| {
            // Minimal: just render blank screen
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
```

**Acceptance Criteria:**
- [x] Terminal switches to alternate screen
- [x] Raw mode enabled
- [x] Pressing 'q' exits cleanly
- [x] Terminal restored to original state on exit
- [x] Ctrl+C handled gracefully (cleanup runs)
- [x] Tests: terminal state before/after

---

### Task 1.3: Theme System
**Dependencies:** 1.1
**Effort:** ~3 hours
**Deliverable:** Theme struct with dark/light variants

```rust
// rex/src/tui/theme.rs
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub border_focused: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub muted: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(30, 30, 46),      // #1e1e2e
            foreground: Color::Rgb(205, 214, 244),   // #cdd6f4
            border: Color::Rgb(69, 71, 90),          // #45475a
            border_focused: Color::Rgb(137, 180, 250), // #89b4fa
            selected_bg: Color::Rgb(49, 50, 68),     // #313244
            selected_fg: Color::Rgb(137, 180, 250),  // #89b4fa
            success: Color::Rgb(166, 227, 161),      // #a6e3a1
            warning: Color::Rgb(249, 226, 175),      // #f9e2af
            error: Color::Rgb(243, 139, 168),        // #f38ba8
            info: Color::Rgb(137, 220, 235),         // #89dceb
            muted: Color::Rgb(108, 112, 134),        // #6c7086
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::Rgb(239, 241, 245),   // #eff1f5
            foreground: Color::Rgb(76, 79, 105),     // #4c4f69
            border: Color::Rgb(172, 176, 190),       // #acb0be
            border_focused: Color::Rgb(30, 102, 245), // #1e66f5
            selected_bg: Color::Rgb(220, 224, 232),  // #dce0e8
            selected_fg: Color::Rgb(30, 102, 245),   // #1e66f5
            success: Color::Rgb(64, 160, 43),        // #40a02b
            warning: Color::Rgb(223, 142, 29),       // #df8e1d
            error: Color::Rgb(210, 15, 57),          // #d20f39
            info: Color::Rgb(4, 165, 229),           // #04a5e5
            muted: Color::Rgb(156, 160, 176),        // #9ca0b0
        }
    }

    // Style helpers
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.foreground)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    pub fn selected_style(&self) -> Style {
        Style::default()
            .bg(self.selected_bg)
            .fg(self.selected_fg)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }
}
```

**Acceptance Criteria:**
- [x] Theme struct with all colors defined
- [x] Dark and light theme variants
- [x] Style helper methods
- [x] Tests: verify color values match design spec
- [x] Document color codes with hex values

---

### Task 1.4: Shell Structure - Layout System
**Dependencies:** 1.2, 1.3
**Effort:** ~4 hours
**Deliverable:** Shell layout calculation working

```rust
// rex/src/tui/shell.rs
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone)]
pub struct ShellLayout {
    pub title_bar: Rect,
    pub context_bar: Option<Rect>,
    pub content: Rect,
    pub status_line: Option<Rect>,
    pub footer: Rect,
}

impl ShellLayout {
    pub fn calculate(area: Rect, has_context: bool, has_status: bool) -> Self {
        let mut constraints = vec![
            Constraint::Length(3), // title bar (border + line + border)
        ];

        if has_context {
            constraints.push(Constraint::Length(1)); // context bar
        }

        constraints.push(Constraint::Min(0)); // content (fills remaining)

        if has_status {
            constraints.push(Constraint::Length(1)); // status line
        }

        constraints.push(Constraint::Length(3)); // footer (border + line + border)

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let mut idx = 0;
        let title_bar = chunks[idx];
        idx += 1;

        let context_bar = if has_context {
            let rect = chunks[idx];
            idx += 1;
            Some(rect)
        } else {
            None
        };

        let content = chunks[idx];
        idx += 1;

        let status_line = if has_status {
            let rect = chunks[idx];
            idx += 1;
            Some(rect)
        } else {
            None
        };

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
```

**Acceptance Criteria:**
- [x] Layout calculation handles all conditional components
- [x] Content area fills remaining space
- [x] Minimum height respected (15 lines)
- [x] Tests: various terminal sizes (24, 40, 15 lines)
- [x] Tests: with/without context bar and status line

---

### Task 1.5: Shell Components - Title Bar
**Dependencies:** 1.4
**Effort:** ~2 hours
**Deliverable:** Title bar rendering

```rust
// rex/src/tui/shell.rs
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

pub struct TitleBar {
    app_name: String,
    registry_name: Option<String>,
}

impl TitleBar {
    pub fn new() -> Self {
        Self {
            app_name: "Rex Registry Explorer".to_string(),
            registry_name: None,
        }
    }

    pub fn set_registry(&mut self, name: String) {
        self.registry_name = Some(name);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(theme.border_style());

        let inner = block.inner(area);

        // Left side: app name
        let mut spans = vec![Span::styled(&self.app_name, theme.title_style())];

        // Right side: registry name
        if let Some(ref registry) = self.registry_name {
            let width = inner.width as usize;
            let left_len = self.app_name.len();
            let right_text = format!("Registry: {}   [r]", registry);
            let right_len = right_text.len();

            if left_len + right_len < width {
                let spacing = width - left_len - right_len;
                spans.push(Span::raw(" ".repeat(spacing)));
                spans.push(Span::styled(
                    format!("Registry: {} ", registry),
                    Style::default().fg(theme.foreground)
                ));
                spans.push(Span::styled(
                    "[r]",
                    Style::default().fg(theme.info)
                ));
            }
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);

        frame.render_widget(block, area);
        frame.render_widget(paragraph, inner);
    }
}
```

**Acceptance Criteria:**
- [x] App name on left
- [x] Registry on right with [r] shortcut
- [x] Proper spacing between left and right
- [x] Handles narrow terminals gracefully
- [x] Tests: rendering with/without registry
- [x] Tests: various widths (60, 80, 120 cols)

---

### Task 1.6: Shell Components - Footer
**Dependencies:** 1.4
**Effort:** ~2 hours
**Deliverable:** Footer rendering

```rust
// rex/src/tui/shell.rs
pub struct Action {
    pub key: String,
    pub description: String,
    pub enabled: bool,
}

pub struct Footer {
    actions: Vec<Action>,
}

impl Footer {
    pub fn new(actions: Vec<Action>) -> Self {
        Self { actions }
    }

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
                Style::default().fg(theme.info)
            } else {
                Style::default().fg(theme.muted)
            };

            let desc_style = if action.enabled {
                Style::default().fg(theme.foreground)
            } else {
                Style::default().fg(theme.muted)
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
```

**Acceptance Criteria:**
- [x] Actions rendered left to right
- [x] Keys highlighted in info color
- [x] Disabled actions grayed out
- [x] Spacing between actions
- [x] Tests: various action counts (3, 5, 8)
- [x] Tests: enabled vs disabled actions

---

### Task 1.7: Event System
**Dependencies:** 1.2
**Effort:** ~3 hours
**Deliverable:** Event handling and routing

```rust
// rex/src/tui/events.rs
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    // Navigation
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    Tab,

    // Actions
    Search,           // /
    Refresh,          // R
    Delete,           // d
    Copy,             // y
    Help,             // ?
    RegistrySelector, // r
    Inspect,          // i

    // Special
    Quit,             // q
    Back,             // Esc, h (vim)
    Char(char),       // For search input

    // System
    Resize(u16, u16),
}

pub struct EventHandler {
    vim_mode: bool,
}

impl EventHandler {
    pub fn new(vim_mode: bool) -> Self {
        Self { vim_mode }
    }

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

    fn handle_key(&self, key: KeyEvent) -> Event {
        match key.code {
            KeyCode::Char('q') => Event::Quit,
            KeyCode::Char('/') => Event::Search,
            KeyCode::Char('r') => Event::RegistrySelector,
            KeyCode::Char('R') => Event::Refresh,
            KeyCode::Char('d') => Event::Delete,
            KeyCode::Char('y') => Event::Copy,
            KeyCode::Char('?') => Event::Help,
            KeyCode::Char('i') => Event::Inspect,

            KeyCode::Up | KeyCode::Char('k') if self.vim_mode => Event::Up,
            KeyCode::Down | KeyCode::Char('j') if self.vim_mode => Event::Down,
            KeyCode::Left | KeyCode::Char('h') if self.vim_mode => Event::Left,
            KeyCode::Right | KeyCode::Char('l') if self.vim_mode => Event::Right,

            KeyCode::Up => Event::Up,
            KeyCode::Down => Event::Down,
            KeyCode::Left => Event::Left,
            KeyCode::Right => Event::Right,

            KeyCode::Enter => Event::Enter,
            KeyCode::Esc => Event::Back,
            KeyCode::Tab => Event::Tab,
            KeyCode::PageUp => Event::PageUp,
            KeyCode::PageDown => Event::PageDown,
            KeyCode::Home => Event::Home,
            KeyCode::End => Event::End,

            KeyCode::Char(c) => Event::Char(c),

            _ => Event::Char('\0'), // Ignore unknown keys
        }
    }
}
```

**Acceptance Criteria:**
- [x] Key events mapped to application events
- [x] Vim mode support (hjkl navigation)
- [x] Special keys handled (Enter, Esc, Tab)
- [x] Resize events captured
- [x] Tests: key mapping for all events
- [x] Tests: vim mode on/off

---

## Phase 2: Core Infrastructure (Weeks 3-4)

### Task 2.1: Application State
**Dependencies:** 1.7
**Effort:** ~4 hours
**Deliverable:** Central app state management

```rust
// rex/src/tui/app.rs
use std::sync::mpsc::{channel, Sender, Receiver};

#[derive(Debug, Clone)]
pub enum View {
    RepositoryList,
    TagList(String),          // repository name
    ImageDetails(String, String), // repository, tag
    RegistrySelector,
    HelpPanel,
}

#[derive(Debug)]
pub enum Message {
    RepositoriesLoaded(Result<Vec<String>>),
    TagsLoaded(String, Result<Vec<String>>),
    ManifestLoaded(String, String, Result<Manifest>),
    Error(String),
}

pub struct App {
    // State
    pub current_view: View,
    pub view_stack: Vec<View>,
    pub should_quit: bool,

    // Registry
    pub current_registry: String,

    // Data (cached)
    pub repositories: Vec<String>,
    pub tags: HashMap<String, Vec<String>>,

    // Communication
    tx: Sender<Message>,
    rx: Receiver<Message>,

    // Config
    pub theme: Theme,
    pub vim_mode: bool,
}

impl App {
    pub fn new(registry: String, theme: Theme, vim_mode: bool) -> Self {
        let (tx, rx) = channel();

        Self {
            current_view: View::RepositoryList,
            view_stack: vec![],
            should_quit: false,
            current_registry: registry,
            repositories: vec![],
            tags: HashMap::new(),
            tx,
            rx,
            theme,
            vim_mode,
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Quit => {
                if self.view_stack.is_empty() {
                    self.should_quit = true;
                } else {
                    self.pop_view();
                }
            }
            Event::Back => {
                self.pop_view();
            }
            _ => {
                // Delegate to current view
                self.handle_view_event(event)?;
            }
        }
        Ok(())
    }

    fn handle_view_event(&mut self, event: Event) -> Result<()> {
        match &self.current_view {
            View::RepositoryList => self.handle_repo_list_event(event),
            View::TagList(_) => self.handle_tag_list_event(event),
            View::ImageDetails(_, _) => self.handle_details_event(event),
            View::RegistrySelector => self.handle_registry_selector_event(event),
            View::HelpPanel => self.handle_help_event(event),
        }
    }

    pub fn push_view(&mut self, view: View) {
        self.view_stack.push(self.current_view.clone());
        self.current_view = view;
    }

    pub fn pop_view(&mut self) {
        if let Some(view) = self.view_stack.pop() {
            self.current_view = view;
        }
    }

    pub fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            self.handle_message(msg);
        }
    }

    fn handle_message(&mut self, msg: Message) {
        // Process messages from worker threads
        match msg {
            Message::RepositoriesLoaded(Ok(repos)) => {
                self.repositories = repos;
            }
            Message::TagsLoaded(repo, Ok(tags)) => {
                self.tags.insert(repo, tags);
            }
            Message::Error(err) => {
                // Show error banner
                eprintln!("Error: {}", err);
            }
            _ => {}
        }
    }

    pub fn spawn_worker<F>(&self, f: F)
    where
        F: FnOnce() -> Message + Send + 'static,
    {
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let msg = f();
            let _ = tx.send(msg);
        });
    }
}
```

**Acceptance Criteria:**
- [x] App state with view stack
- [x] Event routing to current view
- [x] View push/pop navigation
- [x] Message passing setup (tx/rx)
- [x] Worker thread spawning
- [x] Tests: view transitions
- [x] Tests: message handling

---

### Task 2.2: Worker System
**Dependencies:** 2.1
**Effort:** ~3 hours
**Deliverable:** Background I/O workers

```rust
// rex/src/tui/worker.rs
use std::sync::mpsc::Sender;
use librex::Rex;

pub fn fetch_repositories(
    registry_url: String,
    tx: Sender<Message>,
) {
    std::thread::spawn(move || {
        let result = (|| -> Result<Vec<String>> {
            let rex = Rex::connect(&registry_url)?;
            let repos = rex.list_repositories()?;
            Ok(repos.into_iter().map(|r| r.name).collect())
        })();

        let msg = Message::RepositoriesLoaded(result);
        let _ = tx.send(msg);
    });
}

pub fn fetch_tags(
    registry_url: String,
    repository: String,
    tx: Sender<Message>,
) {
    let repo_clone = repository.clone();
    std::thread::spawn(move || {
        let result = (|| -> Result<Vec<String>> {
            let rex = Rex::connect(&registry_url)?;
            let tags = rex.list_tags(&repository)?;
            Ok(tags)
        })();

        let msg = Message::TagsLoaded(repo_clone, result);
        let _ = tx.send(msg);
    });
}

pub fn fetch_manifest(
    registry_url: String,
    repository: String,
    tag: String,
    tx: Sender<Message>,
) {
    let repo_clone = repository.clone();
    let tag_clone = tag.clone();
    std::thread::spawn(move || {
        let result = (|| -> Result<Manifest> {
            let rex = Rex::connect(&registry_url)?;
            let reference = Reference::parse(&format!("{}:{}", repository, tag))?;
            let manifest = rex.get_manifest(&reference)?;
            Ok(manifest)
        })();

        let msg = Message::ManifestLoaded(repo_clone, tag_clone, result);
        let _ = tx.send(msg);
    });
}
```

**Acceptance Criteria:**
- [x] Workers spawn without blocking
- [x] Results sent via channels
- [x] Errors propagated correctly
- [x] Tests: mock Rex client
- [x] Tests: message delivery

---

## Phase 3: Basic Views (Weeks 5-6)

### Task 3.1: Repository List View - Data Model
**Dependencies:** 2.1
**Effort:** ~2 hours
**Deliverable:** State for repository list

```rust
// rex/src/tui/views/repos.rs
#[derive(Debug, Clone)]
pub struct RepositoryItem {
    pub name: String,
    pub tag_count: usize,
    pub total_size: u64,
    pub last_updated: Option<String>,
}

pub struct RepositoryListState {
    pub items: Vec<RepositoryItem>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    pub filter: String,
}

impl RepositoryListState {
    pub fn new() -> Self {
        Self {
            items: vec![],
            selected: 0,
            scroll_offset: 0,
            loading: false,
            filter: String::new(),
        }
    }

    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn selected_item(&self) -> Option<&RepositoryItem> {
        self.items.get(self.selected)
    }

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
}
```

**Acceptance Criteria:**
- [x] State struct with items and selection
- [x] Navigation methods (next, previous)
- [x] Filtering logic
- [x] Tests: selection bounds
- [x] Tests: filtering

---

### Task 3.2: Repository List View - Rendering
**Dependencies:** 3.1
**Effort:** ~4 hours
**Deliverable:** Repository list UI

```rust
// rex/src/tui/views/repos.rs
use ratatui::widgets::{Table, Row, Cell};

impl RepositoryListState {
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let items = self.filtered_items();

        let header = Row::new(vec![
            Cell::from("NAME"),
            Cell::from("TAGS"),
            Cell::from("SIZE"),
            Cell::from("LAST UPDATED"),
        ])
        .style(theme.title_style());

        let rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    theme.selected_style()
                } else {
                    Style::default()
                };

                let indicator = if i == self.selected { "▶ " } else { "  " };

                Row::new(vec![
                    Cell::from(format!("{}{}", indicator, item.name)),
                    Cell::from(item.tag_count.to_string()),
                    Cell::from(format_size(item.total_size)),
                    Cell::from(item.last_updated.as_deref().unwrap_or("-")),
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

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.border_style())
            );

        frame.render_widget(table, area);
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
```

**Acceptance Criteria:**
- [x] Table with 4 columns
- [x] Selected row highlighted
- [x] Indicator (▶) on selected
- [x] Size formatting (KB/MB/GB)
- [x] Scrolling support
- [x] Tests: rendering various item counts
- [x] Tests: size formatting

---

### Task 3.3: Repository List View - Integration
**Dependencies:** 3.2, 2.2
**Effort:** ~3 hours
**Deliverable:** Repository list connected to workers

**Acceptance Criteria:**
- [x] Load repositories on view enter
- [x] Show loading spinner
- [x] Handle Enter key (navigate to tags)
- [x] Handle up/down navigation
- [x] Tests: event handling
- [x] Tests: worker integration

---

### Task 3.4: Tag List View - Data Model
**Dependencies:** 2.1
**Effort:** ~2 hours
**Deliverable:** State for tag list

```rust
// rex/src/tui/views/tags.rs
#[derive(Debug, Clone)]
pub struct TagItem {
    pub tag: String,
    pub digest: String,
    pub size: u64,
    pub platforms: Vec<String>,
    pub updated: Option<String>,
}

pub struct TagListState {
    pub repository: String,
    pub items: Vec<TagItem>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub loading: bool,
}

impl TagListState {
    pub fn new(repository: String) -> Self {
        Self {
            repository,
            items: vec![],
            selected: 0,
            scroll_offset: 0,
            loading: false,
        }
    }

    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn selected_item(&self) -> Option<&TagItem> {
        self.items.get(self.selected)
    }
}
```

**Acceptance Criteria:**
- [x] State struct with tag items
- [x] Navigation methods
- [x] Selected item accessor
- [x] Tests: selection bounds

---

### Task 3.5: Tag List View - Rendering
**Dependencies:** 3.4
**Effort:** ~3 hours
**Deliverable:** Tag list UI with breadcrumb

**Acceptance Criteria:**
- [x] Breadcrumb showing registry › repository
- [x] Table with 5 columns (tag, digest, size, platforms, updated)
- [x] Truncated digest display (first 12 chars)
- [x] Platform summary (e.g., "🐧×3")
- [x] Selected row highlighted
- [x] Tests: rendering

---

### Task 3.6: Tag List View - Integration
**Dependencies:** 3.5, 2.2
**Effort:** ~3 hours
**Deliverable:** Tag list connected to workers

**Acceptance Criteria:**
- [x] Load tags on view enter
- [x] Show loading spinner
- [x] Handle Enter key (navigate to details)
- [x] Handle back navigation (Esc)
- [x] Tests: event handling
- [x] Tests: worker integration

---

### Task 3.7: Image Details View - Basic
**Dependencies:** 2.1
**Effort:** ~4 hours
**Deliverable:** Simple details view

```rust
// rex/src/tui/views/details.rs
pub struct ImageDetailsState {
    pub repository: String,
    pub tag: String,
    pub manifest: Option<Manifest>,
    pub config: Option<ImageConfig>,
    pub scroll_offset: usize,
    pub loading: bool,
}

impl ImageDetailsState {
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if self.loading {
            self.render_loading(frame, area, theme);
            return;
        }

        if let Some(ref manifest) = self.manifest {
            self.render_details(frame, area, manifest, theme);
        }
    }

    fn render_details(&self, frame: &mut Frame, area: Rect, manifest: &Manifest, theme: &Theme) {
        let text = vec![
            Line::from("Overview"),
            Line::from(""),
            Line::from(format!("Type: {}", manifest.media_type)),
            Line::from(format!("Digest: {}", manifest.digest)),
            Line::from(format!("Size: {}", format_size(manifest.size))),
            Line::from(""),
            Line::from("Platforms:"),
            // ... platform details
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.border_style())
            )
            .scroll((self.scroll_offset as u16, 0));

        frame.render_widget(paragraph, area);
    }
}
```

**Acceptance Criteria:**
- [x] Overview section (type, digest, size)
- [x] Scrollable content
- [x] Loading state
- [x] Basic info display
- [x] Tests: rendering with data

---

## Phase 4: Interactive Features (Weeks 7-8)

### Task 4.1: Search/Filter UI
**Dependencies:** 3.2
**Effort:** ~4 hours
**Deliverable:** Search box with live filtering

**Acceptance Criteria:**
- [x] Search box at top of list views
- [x] Real-time filtering as user types
- [x] Highlight matching characters
- [x] Show match count
- [x] Esc to clear search
- [x] Tests: filtering logic

---

### Task 4.2: Loading States & Spinners
**Dependencies:** All views
**Effort:** ~3 hours
**Deliverable:** Consistent loading indicators

**Acceptance Criteria:**
- [x] Animated spinner (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏)
- [x] "Loading..." message
- [x] Background refresh indicator
- [x] Tests: spinner animation

---

### Task 4.3: Status Banners
**Dependencies:** 1.4
**Effort:** ~3 hours
**Deliverable:** Warning/error/success banners

**Acceptance Criteria:**
- [x] Banner types (loading, warning, error, success)
- [x] Auto-dismiss (5s for success)
- [x] Dismissible with [×]
- [x] Multiple banners stack
- [x] Tests: banner lifecycle

---

### Task 4.4: Registry Selector Modal
**Dependencies:** 2.1
**Effort:** ~4 hours
**Deliverable:** Registry switching dialog

**Acceptance Criteria:**
- [x] Modal overlay
- [x] List of configured registries
- [x] Status indicators (online/offline)
- [x] Switch on Enter
- [x] Close on Esc
- [x] Tests: modal rendering

---

### Task 4.5: Help Panel
**Dependencies:** 2.1
**Effort:** ~3 hours
**Deliverable:** Keyboard shortcut reference

**Acceptance Criteria:**
- [x] Modal overlay
- [x] Organized by category
- [x] All keybindings documented
- [x] Context-specific section
- [x] Toggle with ?
- [x] Tests: help content

---

## Phase 5: Polish & Testing (Weeks 9-10)

### Task 5.1: Error Handling
**Dependencies:** All features
**Effort:** ~4 hours
**Deliverable:** Comprehensive error handling

**Acceptance Criteria:**
- [x] Error modals with suggestions
- [x] Non-blocking errors (banners)
- [x] Network error handling
- [x] Empty state messages
- [x] Tests: all error scenarios

---

### Task 5.2: Integration Testing
**Dependencies:** All features
**Effort:** ~8 hours
**Deliverable:** End-to-end tests

**Acceptance Criteria:**
- [x] Full navigation flows
- [x] Search workflows
- [x] Error recovery
- [x] Registry switching
- [x] All views tested together

---

### Task 5.3: Performance Optimization
**Dependencies:** All features
**Effort:** ~4 hours
**Deliverable:** 60 FPS rendering

**Acceptance Criteria:**
- [x] Virtual scrolling for large lists
- [x] Efficient redraws
- [x] Non-blocking I/O verified
- [x] Memory usage profiled
- [x] Tests: performance benchmarks

---

### Task 5.4: Documentation
**Dependencies:** All features
**Effort:** ~4 hours
**Deliverable:** User documentation

**Acceptance Criteria:**
- [x] Usage guide
- [x] Keyboard reference
- [x] Screenshot/demo
- [x] Troubleshooting section

---

## Summary

**Total Estimated Effort:** ~10 weeks (2 developers) or 20 weeks (1 developer)

**Phase Breakdown:**
- Phase 1 (Foundation): 17 hours → ~3 days
- Phase 2 (Infrastructure): 7 hours → ~1 day
- Phase 3 (Basic Views): 21 hours → ~3 days
- Phase 4 (Features): 17 hours → ~2 days
- Phase 5 (Polish): 20 hours → ~3 days

**Total:** ~85 hours → ~12 working days for core implementation

**Critical Path:**
1.1 → 1.2 → 1.7 → 2.1 → 2.2 → 3.1 → 3.2 → 3.3 (Repository list working)

**Parallel Tracks:**
- Theme system (1.3) can be done in parallel with terminal setup
- Views (3.1-3.7) can be split among developers
- Features (4.1-4.5) can be done in parallel after views are done

**Testing Strategy:**
- Unit tests for each component (TDD approach)
- Integration tests after Phase 3
- Manual testing throughout
- Performance benchmarks in Phase 5

**Risk Mitigation:**
- Start with simplest view (repository list)
- Prove non-blocking I/O early (Phase 2)
- User testing after Phase 3 (before polish)
- Buffer time in estimates (20% contingency)
