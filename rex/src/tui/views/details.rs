//! Image details view data model.
//!
//! Provides the data structure and state management for the image details view.

use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use librex::oci::{ImageConfiguration, ImageManifest, ManifestOrIndex};

use crate::tui::theme::Theme;

/// State for the image details view.
#[derive(Debug, Clone)]
pub struct ImageDetailsState {
    /// Repository name
    pub repository: String,
    /// Tag name
    pub tag: String,
    /// Manifest or index data
    pub manifest: Option<ManifestOrIndex>,
    /// Configuration data (for single-platform manifests)
    pub config: Option<ImageConfiguration>,
    /// Scroll offset for content
    pub scroll_offset: usize,
    /// Whether data is currently loading
    pub loading: bool,
}

impl ImageDetailsState {
    /// Create a new image details state.
    ///
    /// # Arguments
    ///
    /// * `repository` - The name of the repository
    /// * `tag` - The name of the tag
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::views::details::ImageDetailsState;
    ///
    /// let state = ImageDetailsState::new("alpine".to_string(), "latest".to_string());
    /// assert_eq!(state.repository, "alpine");
    /// assert_eq!(state.tag, "latest");
    /// ```
    pub fn new(repository: String, tag: String) -> Self {
        Self {
            repository,
            tag,
            manifest: None,
            config: None,
            scroll_offset: 0,
            loading: false,
        }
    }

    /// Scroll down by one line.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::views::details::ImageDetailsState;
    ///
    /// let mut state = ImageDetailsState::new("alpine".to_string(), "latest".to_string());
    /// assert_eq!(state.scroll_offset, 0);
    /// state.scroll_down();
    /// assert_eq!(state.scroll_offset, 1);
    /// ```
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Scroll up by one line.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::tui::views::details::ImageDetailsState;
    ///
    /// let mut state = ImageDetailsState::new("alpine".to_string(), "latest".to_string());
    /// state.scroll_offset = 5;
    /// state.scroll_up();
    /// assert_eq!(state.scroll_offset, 4);
    /// ```
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down by a page (10 lines).
    pub fn scroll_page_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(10);
    }

    /// Scroll up by a page (10 lines).
    pub fn scroll_page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(10);
    }

    /// Scroll to the top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Render the image details view.
    ///
    /// Displays manifest information, configuration, layers, and other metadata.
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render to
    /// * `area` - The rectangular area to render in
    /// * `theme` - The theme to use for styling
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let title = format!(" {} : {} ", self.repository, self.tag);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .title(title);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Show loading state
        if self.loading {
            let loading_text = vec![Line::from("Loading manifest...")];
            let loading = Paragraph::new(loading_text).style(theme.muted_style());
            frame.render_widget(loading, inner);
            return;
        }

        // Show content based on what data we have
        let content = if let Some(manifest_or_index) = &self.manifest {
            self.render_manifest_content(manifest_or_index)
        } else {
            vec![Line::from("No manifest data available")]
        };

        let paragraph = Paragraph::new(content)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset as u16, 0));

        frame.render_widget(paragraph, inner);
    }

    /// Render manifest content based on whether it's a single manifest or index.
    fn render_manifest_content(&self, manifest_or_index: &ManifestOrIndex) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        match manifest_or_index {
            ManifestOrIndex::Manifest(manifest) => {
                lines.extend(self.render_single_manifest(manifest));
            }
            ManifestOrIndex::Index(index) => {
                lines.extend(self.render_manifest_index(index));
            }
        }

        lines
    }

    /// Render a single-platform manifest.
    fn render_single_manifest(&self, manifest: &ImageManifest) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Manifest info
        lines.push(Line::from(Span::styled(
            "MANIFEST",
            ratatui::style::Style::default().bold(),
        )));
        lines.push(Line::from(""));

        lines.push(Line::from(format!(
            "Schema Version: {}",
            manifest.schema_version()
        )));

        if let Some(media_type) = manifest.media_type() {
            lines.push(Line::from(format!("Media Type: {}", media_type)));
        }

        lines.push(Line::from(""));

        // Config
        lines.push(Line::from(Span::styled(
            "CONFIG",
            ratatui::style::Style::default().bold(),
        )));
        lines.push(Line::from(""));

        let config = manifest.config();
        lines.push(Line::from(format!("  Digest: {}", config.digest())));
        lines.push(Line::from(format!(
            "  Size: {}",
            librex::format::format_size(config.size())
        )));
        lines.push(Line::from(format!("  Media Type: {}", config.media_type())));

        lines.push(Line::from(""));

        // Layers
        lines.push(Line::from(Span::styled(
            format!("LAYERS ({})", manifest.layers().len()),
            ratatui::style::Style::default().bold(),
        )));
        lines.push(Line::from(""));

        for (i, layer) in manifest.layers().iter().enumerate() {
            lines.push(Line::from(format!("Layer {}:", i + 1)));
            lines.push(Line::from(format!("  Digest: {}", layer.digest())));
            lines.push(Line::from(format!(
                "  Size: {}",
                librex::format::format_size(layer.size())
            )));
            lines.push(Line::from(format!("  Media Type: {}", layer.media_type())));
            lines.push(Line::from(""));
        }

        // Configuration details (if available)
        if let Some(config) = &self.config {
            lines.push(Line::from(Span::styled(
                "CONFIGURATION",
                ratatui::style::Style::default().bold(),
            )));
            lines.push(Line::from(""));

            lines.push(Line::from(format!(
                "Architecture: {}",
                config.architecture()
            )));

            lines.push(Line::from(format!("OS: {}", config.os())));

            if let Some(config_inner) = config.config() {
                if let Some(env) = config_inner.env()
                    && !env.is_empty()
                {
                    lines.push(Line::from(""));
                    lines.push(Line::from("Environment:"));
                    for var in env {
                        lines.push(Line::from(format!("  {}", var)));
                    }
                }

                if let Some(cmd) = config_inner.cmd()
                    && !cmd.is_empty()
                {
                    lines.push(Line::from(""));
                    lines.push(Line::from(format!("Cmd: {}", cmd.join(" "))));
                }

                if let Some(entrypoint) = config_inner.entrypoint()
                    && !entrypoint.is_empty()
                {
                    lines.push(Line::from(format!("Entrypoint: {}", entrypoint.join(" "))));
                }

                if let Some(working_dir) = config_inner.working_dir() {
                    lines.push(Line::from(format!("Working Dir: {}", working_dir)));
                }

                if let Some(user) = config_inner.user() {
                    lines.push(Line::from(format!("User: {}", user)));
                }
            }

            // History
            if let Some(history) = config.history()
                && !history.is_empty()
            {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("HISTORY ({} entries)", history.len()),
                    ratatui::style::Style::default().bold(),
                )));
                lines.push(Line::from(""));

                for (i, entry) in history.iter().take(10).enumerate() {
                    lines.push(Line::from(format!(
                        "{}. {}",
                        i + 1,
                        entry.created_by().as_deref().unwrap_or("(no command)")
                    )));
                }

                if history.len() > 10 {
                    lines.push(Line::from(format!("... and {} more", history.len() - 10)));
                }
            }
        }

        lines
    }

    /// Render a multi-platform manifest index.
    fn render_manifest_index(&self, index: &librex::oci::ImageIndex) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            "MULTI-PLATFORM IMAGE INDEX",
            ratatui::style::Style::default().bold(),
        )));
        lines.push(Line::from(""));

        lines.push(Line::from(format!(
            "Schema Version: {}",
            index.schema_version()
        )));

        if let Some(media_type) = index.media_type() {
            lines.push(Line::from(format!("Media Type: {}", media_type)));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("PLATFORMS ({})", index.manifests().len()),
            ratatui::style::Style::default().bold(),
        )));
        lines.push(Line::from(""));

        for manifest in index.manifests() {
            if let Some(platform) = manifest.platform() {
                let variant_str = platform
                    .variant()
                    .as_ref()
                    .map(|v| format!("/{}", v))
                    .unwrap_or_default();
                lines.push(Line::from(format!(
                    "• {}/{}{}",
                    platform.os(),
                    platform.architecture(),
                    variant_str
                )));
                lines.push(Line::from(format!("  Digest: {}", manifest.digest())));
                lines.push(Line::from(format!(
                    "  Size: {}",
                    librex::format::format_size(manifest.size())
                )));
                lines.push(Line::from(""));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(
            "(Select a specific platform to view detailed manifest)",
        ));

        lines
    }
}

impl Default for ImageDetailsState {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

#[cfg(test)]
#[path = "details_tests.rs"]
mod tests;
