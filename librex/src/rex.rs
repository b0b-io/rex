//! High-level API for Rex library.
//!
//! This module provides a simplified, user-friendly interface for interacting with
//! OCI registries. It's the recommended entry point for most users.
//!
//! # Examples
//!
//! ```no_run
//! use librex::Rex;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to a registry
//!     let mut rex = Rex::connect("http://localhost:5000").await?;
//!
//!     // List all repositories
//!     let repos = rex.list_repositories().await?;
//!     for repo in repos {
//!         println!("{}", repo);
//!     }
//!
//!     // List tags for a repository
//!     let tags = rex.list_tags("alpine").await?;
//!     for tag in tags {
//!         println!("{}", tag);
//!     }
//!
//!     // Search for repositories
//!     let results = rex.search_repositories("alp").await?;
//!     for result in results {
//!         println!("{}", result.value);
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::auth::Credentials;
use crate::cache::Cache;
use crate::client::Client;
use crate::config::Config;
use crate::digest::Digest;
use crate::error::Result;
use crate::reference::Reference;
use crate::registry::Registry;
use crate::search::{SearchResult, search_images, search_repositories, search_tags};
use oci_spec::image::ImageManifest;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// High-level interface for interacting with OCI registries.
///
/// `Rex` provides a simplified API that handles common workflows and hides
/// implementation complexity. It orchestrates the client, registry, authentication,
/// cache, and search modules.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```no_run
/// use librex::Rex;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut rex = Rex::connect("http://localhost:5000").await?;
///     let repos = rex.list_repositories().await?;
///     println!("Found {} repositories", repos.len());
///     Ok(())
/// }
/// ```
///
/// ## With Authentication
///
/// ```no_run
/// use librex::{Rex, Credentials};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut rex = Rex::connect("https://registry.example.com").await?;
///
///     let creds = Credentials::Basic {
///         username: "user".to_string(),
///         password: "pass".to_string(),
///     };
///     rex.login(creds);
///
///     let repos = rex.list_repositories().await?;
///     Ok(())
/// }
/// ```
///
/// ## With Caching
///
/// ```no_run
/// use librex::Rex;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut rex = Rex::builder()
///         .registry_url("http://localhost:5000")
///         .with_cache("/tmp/rex-cache")
///         .build()
///         .await?;
///
///     let repos = rex.list_repositories().await?;
///     Ok(())
/// }
/// ```
pub struct Rex {
    /// The underlying registry client.
    registry: Registry,
    /// Registry URL for reference.
    registry_url: String,
    /// Cached list of repositories for search operations.
    cached_repositories: Option<Vec<String>>,
    /// Cached tags per repository for search operations.
    cached_tags: HashMap<String, Vec<String>>,
}

impl Rex {
    /// Connect to a registry with default settings.
    ///
    /// This is the simplest way to get started. It creates a connection to the
    /// specified registry without caching or authentication.
    ///
    /// # Arguments
    ///
    /// * `registry_url` - The registry URL (e.g., "http://localhost:5000")
    ///
    /// # Returns
    ///
    /// A connected `Rex` instance ready to use.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("http://localhost:5000").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(registry_url: &str) -> Result<Self> {
        let client = Client::new(registry_url)?;
        let registry = Registry::new(client, None, None);

        Ok(Self {
            registry,
            registry_url: registry_url.to_string(),
            cached_repositories: None,
            cached_tags: HashMap::new(),
        })
    }

    /// Create a builder for advanced configuration.
    ///
    /// Use this when you need more control over caching, authentication,
    /// or other settings.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::builder()
    ///         .registry_url("http://localhost:5000")
    ///         .with_cache("/tmp/rex-cache")
    ///         .build()
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn builder() -> RexBuilder {
        RexBuilder::new()
    }

    /// Verify that the registry is accessible and supports OCI Distribution Specification.
    ///
    /// This performs a version check by calling the `/v2/` endpoint.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("http://localhost:5000").await?;
    ///     rex.check().await?;
    ///     println!("Registry is accessible!");
    ///     Ok(())
    /// }
    /// ```
    pub async fn check(&mut self) -> Result<()> {
        self.registry.check_version().await
    }

    /// Set credentials for authenticated requests.
    ///
    /// # Arguments
    ///
    /// * `credentials` - The credentials to use for authentication
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::{Rex, Credentials};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("https://registry.example.com").await?;
    ///
    ///     rex.login(Credentials::Basic {
    ///         username: "user".to_string(),
    ///         password: "pass".to_string(),
    ///     });
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn login(&mut self, credentials: Credentials) {
        self.registry.set_credentials(credentials);
    }

    /// Clear credentials and switch to anonymous access.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("http://localhost:5000").await?;
    ///     rex.logout();
    ///     Ok(())
    /// }
    /// ```
    pub fn logout(&mut self) {
        self.registry.clear_credentials();
    }

    /// List all repositories in the registry.
    ///
    /// This fetches the catalog of repositories. Results are cached internally
    /// for use by search operations.
    ///
    /// # Returns
    ///
    /// A vector of repository names.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("http://localhost:5000").await?;
    ///
    ///     let repos = rex.list_repositories().await?;
    ///     for repo in repos {
    ///         println!("{}", repo);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn list_repositories(&mut self) -> Result<Vec<String>> {
        let repos = self.registry.list_repositories().await?;
        self.cached_repositories = Some(repos.clone());
        Ok(repos)
    }

    /// List all tags for a specific repository.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository name (e.g., "alpine")
    ///
    /// # Returns
    ///
    /// A vector of tag names for the repository.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("http://localhost:5000").await?;
    ///
    ///     let tags = rex.list_tags("alpine").await?;
    ///     for tag in tags {
    ///         println!("{}", tag);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn list_tags(&mut self, repository: &str) -> Result<Vec<String>> {
        let tags = self.registry.list_tags(repository).await?;
        self.cached_tags
            .insert(repository.to_string(), tags.clone());
        Ok(tags)
    }

    /// Get the manifest for a specific image.
    ///
    /// # Arguments
    ///
    /// * `image` - The image reference (e.g., "alpine:latest" or "alpine@sha256:...")
    ///
    /// # Returns
    ///
    /// The image manifest containing layer and configuration information.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("http://localhost:5000").await?;
    ///
    ///     let manifest = rex.get_manifest("alpine:latest").await?;
    ///     println!("Layers: {}", manifest.layers().len());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_manifest(&mut self, image: &str) -> Result<ImageManifest> {
        let reference = image.parse::<Reference>()?;
        self.registry.get_manifest(&reference).await
    }

    /// Get a blob (layer or config) by digest.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository name
    /// * `digest` - The content digest (e.g., "sha256:abc123...")
    ///
    /// # Returns
    ///
    /// The raw blob content as bytes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("http://localhost:5000").await?;
    ///
    ///     let digest = "sha256:abc123...".parse()?;
    ///     let blob = rex.get_blob("alpine", &digest).await?;
    ///     println!("Blob size: {} bytes", blob.len());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_blob(&mut self, repository: &str, digest: &Digest) -> Result<Vec<u8>> {
        self.registry.get_blob(repository, digest).await
    }

    /// Search for repositories by name using fuzzy matching.
    ///
    /// This uses an fzf-like fuzzy matching algorithm. It automatically fetches
    /// the repository catalog if not already cached.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query (e.g., "alp" to match "alpine")
    ///
    /// # Returns
    ///
    /// A vector of search results sorted by relevance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("http://localhost:5000").await?;
    ///
    ///     let results = rex.search_repositories("alp").await?;
    ///     for result in results {
    ///         println!("{} (score: {})", result.value, result.score);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn search_repositories(&mut self, query: &str) -> Result<Vec<SearchResult>> {
        // Fetch repositories if not cached
        if self.cached_repositories.is_none() {
            self.list_repositories().await?;
        }

        let repos = self.cached_repositories.as_ref().unwrap();
        Ok(search_repositories(query, repos))
    }

    /// Search for tags within a specific repository using fuzzy matching.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository name
    /// * `query` - The search query (e.g., "lat" to match "latest")
    ///
    /// # Returns
    ///
    /// A vector of search results sorted by relevance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("http://localhost:5000").await?;
    ///
    ///     let results = rex.search_tags("alpine", "lat").await?;
    ///     for result in results {
    ///         println!("{}", result.value);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn search_tags(
        &mut self,
        repository: &str,
        query: &str,
    ) -> Result<Vec<SearchResult>> {
        // Fetch tags if not cached
        if !self.cached_tags.contains_key(repository) {
            self.list_tags(repository).await?;
        }

        let tags = self.cached_tags.get(repository).unwrap();
        Ok(search_tags(query, tags))
    }

    /// Search for images (repository:tag combinations) using fuzzy matching.
    ///
    /// This searches across both repository names and tags. If the query contains
    /// a colon (e.g., "alp:lat"), it searches repositories and tags separately.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query (e.g., "alp" or "alp:lat")
    ///
    /// # Returns
    ///
    /// A vector of search results with full image references (repository:tag).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::Rex;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut rex = Rex::connect("http://localhost:5000").await?;
    ///
    ///     // Search for repositories matching "alp"
    ///     let results = rex.search_images("alp").await?;
    ///
    ///     // Search for repositories matching "alp" with tags matching "lat"
    ///     let results = rex.search_images("alp:lat").await?;
    ///
    ///     for result in results {
    ///         println!("{}", result.value);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn search_images(&mut self, query: &str) -> Result<Vec<SearchResult>> {
        // Fetch repositories if not cached
        if self.cached_repositories.is_none() {
            self.list_repositories().await?;
        }

        // Clone repos list to avoid borrow conflicts
        let repos = self.cached_repositories.as_ref().unwrap().clone();

        // Fetch tags for all repositories if not cached
        for repo in &repos {
            if !self.cached_tags.contains_key(repo)
                && let Ok(tags) = self.list_tags(repo).await
            {
                self.cached_tags.insert(repo.clone(), tags);
            }
        }

        Ok(search_images(query, &repos, &self.cached_tags))
    }

    /// Get the registry URL.
    pub fn registry_url(&self) -> &str {
        &self.registry_url
    }
}

/// Builder for creating a `Rex` instance with custom configuration.
///
/// # Examples
///
/// ```no_run
/// use librex::Rex;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut rex = Rex::builder()
///         .registry_url("http://localhost:5000")
///         .with_cache("/tmp/rex-cache")
///         .with_config_file("./config.toml")
///         .build()
///         .await?;
///     Ok(())
/// }
/// ```
pub struct RexBuilder {
    registry_url: Option<String>,
    cache_dir: Option<PathBuf>,
    config: Option<Config>,
    credentials: Option<Credentials>,
}

impl RexBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            registry_url: None,
            cache_dir: None,
            config: None,
            credentials: None,
        }
    }

    /// Set the registry URL.
    pub fn registry_url(mut self, url: &str) -> Self {
        self.registry_url = Some(url.to_string());
        self
    }

    /// Enable caching with the specified directory.
    pub fn with_cache(mut self, cache_dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(cache_dir.into());
        self
    }

    /// Load configuration from a file.
    pub fn with_config_file(mut self, path: impl Into<PathBuf>) -> Self {
        // Load config from file
        if let Ok(content) = std::fs::read_to_string(path.into())
            && let Ok(config) = Config::from_yaml_str(&content)
        {
            self.config = Some(config);
        }
        self
    }

    /// Set configuration directly.
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Set credentials for authentication.
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Build the `Rex` instance.
    pub async fn build(self) -> Result<Rex> {
        let registry_url = self
            .registry_url
            .ok_or_else(|| crate::error::RexError::validation("Registry URL is required"))?;

        let client = Client::new(&registry_url)?;

        // Create cache if specified
        let cache = if let Some(cache_dir) = self.cache_dir {
            let config = self.config.unwrap_or_default();
            let capacity = NonZeroUsize::new(config.cache.limits.memory_entries).unwrap();
            Some(Cache::new(cache_dir, config.cache.ttl, capacity))
        } else {
            None
        };

        let registry = Registry::new(client, cache, self.credentials);

        Ok(Rex {
            registry,
            registry_url,
            cached_repositories: None,
            cached_tags: HashMap::new(),
        })
    }
}

impl Default for RexBuilder {
    fn default() -> Self {
        Self::new()
    }
}
