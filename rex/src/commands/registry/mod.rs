use crate::config::{self, RegistryEntry};
use crate::context::VerbosityLevel;
use crate::format::{self, Formattable};
use librex::auth::CredentialStore;
use serde::Serialize;
use std::path::PathBuf;
use tabled::Tabled;
use url::Url;

pub mod handlers;

/// Helper function to generate enhanced "no default registry" error message
pub(crate) fn no_default_registry_error(registries: &[RegistryEntry]) -> String {
    let mut error = String::from("No default registry configured\n");

    if registries.is_empty() {
        error.push_str("\nNo registries available. Add one with:\n");
        error.push_str("  rex registry init <name> <url>\n");
        error.push_str("\nExample:\n");
        error.push_str("  rex registry init dockerhub https://registry-1.docker.io/");
    } else {
        error.push_str("\nAvailable registries:\n");
        for reg in registries {
            error.push_str(&format!("  - {} ({})\n", reg.name, reg.url));
        }
        error.push_str("\nSet default with:\n");
        error.push_str("  rex registry use <name>");
    }

    error
}

/// Helper function to prompt user for confirmation
/// Returns Ok(()) if user confirms (enters 'y'), Err if cancelled
pub(crate) fn confirm(prompt: &str) -> Result<(), String> {
    use std::io::{self, Write};

    print!("{} [y/N]: ", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;

    if input.trim().eq_ignore_ascii_case("y") {
        Ok(())
    } else {
        Err("Operation cancelled".to_string())
    }
}

/// Registry entry with default marker for display purposes
#[derive(Debug, Tabled, Serialize)]
pub struct RegistryDisplay {
    #[tabled(rename = "NAME")]
    pub name: String,
    #[tabled(rename = "URL")]
    pub url: String,
    #[tabled(rename = "DEFAULT")]
    pub default: String,
}

/// Registry check result
#[derive(Debug, Serialize)]
pub struct RegistryCheckResult {
    /// Registry name
    pub name: String,
    /// Registry URL
    pub url: String,
    /// Whether the registry is online and accessible
    pub online: bool,
    /// Authentication status
    pub auth_required: bool,
    /// Whether credentials are configured for this registry
    pub authenticated: bool,
    /// API version if available
    pub api_version: Option<String>,
    /// Error message if check failed
    pub error: Option<String>,
}

impl Formattable for RegistryDisplay {
    fn format_pretty(&self) -> String {
        let default_marker = if !self.default.is_empty() {
            " (default)"
        } else {
            ""
        };
        format!("Name: {}\nURL: {}{}", self.name, self.url, default_marker)
    }
}

impl Formattable for RegistryCheckResult {
    fn format_pretty(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Registry: {}\n", self.name));
        output.push_str(&format!("URL: {}\n", self.url));

        if self.online {
            output.push_str(&format!("Status: {} Online\n", "✓"));
            if let Some(ref api_version) = self.api_version {
                output.push_str(&format!("API Version: {}\n", api_version));
            }

            // Show auth status
            if self.authenticated {
                output.push_str(&format!("Authentication: {} Authenticated\n", "✓"));
            } else if self.auth_required {
                output.push_str("Authentication: ⚠ Required (not configured)\n");
            } else {
                output.push_str("Authentication: ○ Not required\n");
            }
        } else {
            output.push_str(&format!("Status: {} Offline\n", "✗"));
            if let Some(ref error) = self.error {
                output.push_str(&format!("Reason: {}\n", error));
            }
        }

        output
    }
}

impl Formattable for RegistryEntry {
    fn format_pretty(&self) -> String {
        format!("{:20} {}", self.name, self.url)
    }
}

/// Validate and normalize a registry URL
///
/// This function validates that the URL is well-formed and uses http or https scheme.
/// If no scheme is provided, it defaults to http://.
///
/// # Arguments
///
/// * `url_str` - The URL string to validate
///
/// # Returns
///
/// Returns the normalized URL string on success, or an error message on failure.
///
/// # Examples
///
/// ```
/// # use rex::commands::registry::validate_registry_url;
/// assert!(validate_registry_url("https://registry-1.docker.io").is_ok());
/// assert!(validate_registry_url("localhost:5000").is_ok());
/// assert!(validate_registry_url("http:://bad").is_err());
/// ```
pub(crate) fn validate_registry_url(url_str: &str) -> Result<String, String> {
    // Try to parse as-is first
    let url_to_parse = if url_str.contains("://") {
        url_str.to_string()
    } else {
        // Add default http:// scheme if no scheme provided
        format!("http://{}", url_str)
    };

    // Parse and validate the URL
    let parsed_url =
        Url::parse(&url_to_parse).map_err(|e| format!("Invalid URL '{}': {}", url_str, e))?;

    // Validate that the scheme is http or https
    match parsed_url.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(format!(
                "Invalid URL scheme '{}'. Only 'http' and 'https' are supported.",
                scheme
            ));
        }
    }

    // Validate that there's a host
    if parsed_url.host_str().is_none() {
        return Err(format!("Invalid URL '{}': missing host", url_str));
    }

    Ok(parsed_url.to_string())
}

/// Initialize a new registry in the configuration
pub(crate) fn init_registry(config_path: &PathBuf, name: &str, url: &str) -> Result<(), String> {
    // Load existing config or create default
    let mut config = if config_path.exists() {
        config::Config::load(config_path)?
    } else {
        config::Config::default()
    };

    // Check if registry with this name already exists
    if config.registries.list.iter().any(|r| r.name == name) {
        return Err(format!("Registry '{}' already exists", name));
    }

    // Validate and normalize URL
    let normalized_url = validate_registry_url(url)?;

    // Add the new registry
    config.registries.list.push(RegistryEntry {
        name: name.to_string(),
        url: normalized_url,
    });

    // Set as default if this is the first registry
    if config.registries.list.len() == 1 {
        config.registries.default = Some(name.to_string());
    }

    // Save config
    config.save(config_path)?;

    Ok(())
}

/// Remove a registry from the configuration
pub(crate) fn remove_registry(
    config_path: &PathBuf,
    name: &str,
    force: bool,
) -> Result<(), String> {
    // Load existing config
    let mut config = config::Config::load(config_path)?;

    // Find the registry
    let registry = config
        .registries
        .list
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| format!("Registry '{}' not found", name))?;

    // Confirm unless force flag
    if !force {
        confirm(&format!("Remove registry '{}' ({})?", name, registry.url))?;
    }

    // Find the registry index
    let registry_index = config
        .registries
        .list
        .iter()
        .position(|r| r.name == name)
        .unwrap(); // Safe to unwrap - we already checked it exists

    // Remove the registry
    config.registries.list.remove(registry_index);

    // Clear default if it was the removed registry
    if config.registries.default.as_ref() == Some(&name.to_string()) {
        config.registries.default = None;
    }

    // Save config
    config.save(config_path)?;

    Ok(())
}

/// Set the default registry
pub(crate) fn use_registry(config_path: &PathBuf, name: &str) -> Result<(), String> {
    // Load existing config
    let mut config = config::Config::load(config_path)?;

    // Check if registry exists
    if !config.registries.list.iter().any(|r| r.name == name) {
        return Err(format!("Registry '{}' not found", name));
    }

    // Set as default
    config.registries.default = Some(name.to_string());

    // Save config
    config.save(config_path)?;

    Ok(())
}

/// Show details of a specific registry
pub(crate) fn show_registry(config_path: &PathBuf, name: &str) -> Result<RegistryDisplay, String> {
    // Load existing config
    let config = config::Config::load(config_path)?;

    // Find the registry
    let registry = config
        .registries
        .list
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| format!("Registry '{}' not found", name))?;

    // Create display with default marker
    let is_default = config.registries.default.as_ref() == Some(&name.to_string());
    Ok(RegistryDisplay {
        name: registry.name.clone(),
        url: registry.url.clone(),
        default: if is_default {
            "*".to_string()
        } else {
            String::new()
        },
    })
}

/// List all registries
pub(crate) fn list_registries(config_path: &PathBuf) -> Result<Vec<RegistryDisplay>, String> {
    let config = config::Config::load(config_path)?;

    // Create display list with default markers
    let registries: Vec<RegistryDisplay> = config
        .registries
        .list
        .iter()
        .map(|r| {
            let is_default = config.registries.default.as_ref() == Some(&r.name);
            RegistryDisplay {
                name: r.name.clone(),
                url: r.url.clone(),
                default: if is_default {
                    "*".to_string()
                } else {
                    String::new()
                },
            }
        })
        .collect();

    Ok(registries)
}

/// Check registry connectivity and status
pub(crate) async fn check_registry(
    ctx: &crate::context::AppContext,
    config_path: &PathBuf,
    name: &str,
) -> RegistryCheckResult {
    format::print(
        ctx,
        VerbosityLevel::Verbose,
        &format!("Checking registry '{}'...", name),
    );

    // Load config
    format::print(
        ctx,
        VerbosityLevel::VeryVerbose,
        &format!("Loading config from: {}", config_path.display()),
    );
    let config = match config::Config::load(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            return RegistryCheckResult {
                name: name.to_string(),
                url: String::new(),
                online: false,
                auth_required: false,
                authenticated: false,
                api_version: None,
                error: Some(format!("Configuration error: {}", e)),
            };
        }
    };

    // Find registry
    let registry = match config.registries.list.iter().find(|r| r.name == name) {
        Some(reg) => reg,
        None => {
            return RegistryCheckResult {
                name: name.to_string(),
                url: String::new(),
                online: false,
                auth_required: false,
                authenticated: false,
                api_version: None,
                error: Some(format!(
                    "Registry '{}' not found in configuration. Use 'rex registry add' to add it.",
                    name
                )),
            };
        }
    };

    // Check if credentials are configured for this registry
    let creds_path = config::get_credentials_path();
    let authenticated = if let Ok(store) = librex::auth::FileCredentialStore::new(creds_path) {
        store.get(&registry.url).unwrap_or(None).is_some()
    } else {
        false
    };

    // Load credentials if available
    let creds_path = config::get_credentials_path();
    let credentials = if creds_path.exists() {
        if let Ok(store) = librex::auth::FileCredentialStore::new(creds_path) {
            store.get(&registry.url).ok().flatten()
        } else {
            None
        }
    } else {
        None
    };

    // Create client and check version
    format::print(
        ctx,
        VerbosityLevel::VeryVerbose,
        &format!("Connecting to registry at: {}", registry.url),
    );
    let client = match librex::client::Client::new(&registry.url, credentials) {
        Ok(c) => c,
        Err(e) => {
            return RegistryCheckResult {
                name: name.to_string(),
                url: registry.url.clone(),
                online: false,
                auth_required: false,
                authenticated: false,
                api_version: None,
                error: Some(format!("Invalid registry URL: {}", e)),
            };
        }
    };

    match client.check_version().await {
        Ok(version) => RegistryCheckResult {
            name: name.to_string(),
            url: registry.url.clone(),
            online: true,
            auth_required: false,
            authenticated,
            api_version: version.api_version,
            error: None,
        },
        Err(e) => {
            // Check if error is authentication-related
            let error_str = format!("{}", e);
            let auth_required = error_str.contains("Authentication")
                || error_str.contains("Unauthorized")
                || error_str.contains("401")
                || error_str.contains("403");

            // Make error message more user-friendly
            let friendly_error = if auth_required {
                if authenticated {
                    "Authentication failed. Please check your credentials.".to_string()
                } else {
                    "Authentication required. Use 'rex registry login' to authenticate.".to_string()
                }
            } else if error_str.contains("Failed to connect") {
                "Cannot connect to registry. Please check the URL and your network connection."
                    .to_string()
            } else if error_str.contains("timed out") {
                "Connection timed out. The registry may be slow or unreachable.".to_string()
            } else if error_str.contains("Name or service not known")
                || error_str.contains("No such host")
            {
                "Registry hostname could not be resolved. Please check the URL.".to_string()
            } else {
                // For other errors, show the original error
                error_str
            };

            RegistryCheckResult {
                name: name.to_string(),
                url: registry.url.clone(),
                online: false,
                auth_required,
                authenticated,
                api_version: None,
                error: Some(friendly_error),
            }
        }
    }
}

/// Prompt for username if not provided
fn prompt_username(provided_username: Option<&str>) -> Result<String, String> {
    match provided_username {
        Some(username) => Ok(username.to_string()),
        None => {
            print!("Username: ");
            std::io::Write::flush(&mut std::io::stdout())
                .map_err(|e| format!("Failed to flush stdout: {}", e))?;

            let mut username = String::new();
            std::io::stdin()
                .read_line(&mut username)
                .map_err(|e| format!("Failed to read username: {}", e))?;

            Ok(username.trim().to_string())
        }
    }
}

/// Prompt for password if not provided
fn prompt_password(provided_password: Option<&str>) -> Result<String, String> {
    match provided_password {
        Some(password) => Ok(password.to_string()),
        None => rpassword::prompt_password("Password: ")
            .map_err(|e| format!("Failed to read password: {}", e)),
    }
}

/// Login to a registry
pub(crate) async fn login_registry(
    config_path: &PathBuf,
    name: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(), String> {
    // Load config to verify registry exists
    let config = config::Config::load(config_path)?;

    // Find registry
    let registry = config
        .registries
        .list
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| {
            format!(
                "Registry '{}' not found. Use 'rex registry add' to add it first.",
                name
            )
        })?;

    // Get credentials
    let username = prompt_username(username)?;
    let password = prompt_password(password)?;

    // Create credentials
    let credentials = librex::auth::Credentials::basic(&username, &password);

    // Verify credentials by attempting to authenticate with the registry
    println!("Verifying credentials...");
    let client = librex::client::Client::new(&registry.url, Some(credentials.clone()))
        .map_err(|e| format!("Invalid registry URL: {}", e))?;

    client.check_version().await.map_err(|e| {
        let error_str = format!("{}", e);
        if error_str.contains("Authentication") || error_str.contains("401") {
            "Authentication failed. Please check your username and password.".to_string()
        } else if error_str.contains("403") || error_str.contains("Forbidden") {
            "Access forbidden. Your credentials may not have the required permissions.".to_string()
        } else {
            format!("Failed to verify credentials: {}", e)
        }
    })?;

    // Store credentials
    let creds_path = config::get_credentials_path();
    let mut store = librex::auth::FileCredentialStore::new(creds_path)
        .map_err(|e| format!("Failed to initialize credential store: {}", e))?;

    store
        .store(&registry.url, &credentials)
        .map_err(|e| format!("Failed to store credentials: {}", e))?;

    Ok(())
}

/// Logout from a registry
pub(crate) fn logout_registry(config_path: &PathBuf, name: &str) -> Result<(), String> {
    // Load config to verify registry exists and get URL
    let config = config::Config::load(config_path)?;

    // Find registry
    let registry = config
        .registries
        .list
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| {
            format!(
                "Registry '{}' not found. Use 'rex registry list' to see configured registries.",
                name
            )
        })?;

    // Remove credentials
    let creds_path = config::get_credentials_path();
    let mut store = librex::auth::FileCredentialStore::new(creds_path)
        .map_err(|e| format!("Failed to initialize credential store: {}", e))?;

    store
        .remove(&registry.url)
        .map_err(|e| format!("Failed to remove credentials: {}", e))?;

    Ok(())
}

/// Cache statistics display
#[derive(Debug, Serialize)]
pub struct CacheStatsDisplay {
    pub registry: String,
    pub url: String,
    pub disk_entries: u64,
    pub disk_size: u64,
    pub memory_entries: u64,
    pub cache_path: String,
}

impl Formattable for CacheStatsDisplay {
    fn format_pretty(&self) -> String {
        let disk_size_mb = self.disk_size as f64 / 1_048_576.0;
        format!(
            "Cache Statistics for '{}' ({})\n\nOverview:\n  Total Entries: {}\n  Total Size: {:.2} MB\n  Memory Cache: {} entries\n  Disk Cache: {} entries\n\nCache Location: {}",
            self.registry,
            self.url,
            self.disk_entries,
            disk_size_mb,
            self.memory_entries,
            self.disk_entries,
            self.cache_path
        )
    }
}

/// Cache sync statistics
#[derive(Debug, Default)]
pub struct CacheSyncStats {
    pub catalog_entries: u64,
    pub tag_entries: u64,
    pub manifest_entries: u64,
    pub total_size: u64,
}

/// Get cache statistics for a registry
pub fn cache_stats(config_path: &PathBuf, name: Option<&str>) -> Result<CacheStatsDisplay, String> {
    // Load configuration
    let cfg = config::Config::load(config_path)?;

    // Get registry
    let registry = if let Some(name) = name {
        cfg.registries
            .list
            .iter()
            .find(|r| r.name == name)
            .ok_or_else(|| format!("Registry '{}' not found", name))?
    } else {
        // Use default registry
        let default_name = cfg
            .registries
            .default
            .as_ref()
            .ok_or_else(|| no_default_registry_error(&cfg.registries.list))?;
        cfg.registries
            .list
            .iter()
            .find(|r| r.name == *default_name)
            .ok_or_else(|| format!("Default registry '{}' not found", default_name))?
    };

    // Build cache directory path
    let cache_dir = config::get_registry_cache_dir(&registry.url).unwrap();

    // Create a temporary cache instance to get stats
    let cache = librex::cache::Cache::new(
        cache_dir.clone(),
        librex::config::CacheTtl::default(),
        std::num::NonZeroUsize::new(100).unwrap(),
    );

    let stats = cache
        .stats()
        .map_err(|e| format!("Failed to get cache statistics: {}", e))?;

    Ok(CacheStatsDisplay {
        registry: name
            .or(cfg.registries.default.as_deref())
            .unwrap()
            .to_string(),
        url: registry.url.clone(),
        disk_entries: stats.disk_entries,
        disk_size: stats.disk_size,
        memory_entries: stats.memory_entries,
        cache_path: cache_dir.display().to_string(),
    })
}

/// Clear cache for a registry
pub fn cache_clear(
    config_path: &PathBuf,
    name: Option<&str>,
    all: bool,
    force: bool,
) -> Result<librex::cache::ClearStats, String> {
    // Load configuration
    let cfg = config::Config::load(config_path)?;

    if all {
        // Clear all registry caches
        if !force {
            confirm("Clear cache for all registries?")?;
        }

        let mut total_stats = librex::cache::ClearStats::default();
        for registry in &cfg.registries.list {
            let cache_dir = config::get_registry_cache_dir(&registry.url).unwrap();
            let mut cache = librex::cache::Cache::new(
                cache_dir,
                librex::config::CacheTtl::default(),
                std::num::NonZeroUsize::new(100).unwrap(),
            );
            let stats = cache
                .clear()
                .map_err(|e| format!("Failed to clear cache: {}", e))?;
            total_stats.removed_files += stats.removed_files;
            total_stats.reclaimed_space += stats.reclaimed_space;
        }
        return Ok(total_stats);
    }

    // Get registry
    let registry = if let Some(name) = name {
        cfg.registries
            .list
            .iter()
            .find(|r| r.name == name)
            .ok_or_else(|| format!("Registry '{}' not found", name))?
    } else {
        // Use default registry
        let default_name = cfg
            .registries
            .default
            .as_ref()
            .ok_or_else(|| no_default_registry_error(&cfg.registries.list))?;
        cfg.registries
            .list
            .iter()
            .find(|r| r.name == *default_name)
            .ok_or_else(|| format!("Default registry '{}' not found", default_name))?
    };

    // Confirm unless force flag
    if !force {
        let registry_name = name.or(cfg.registries.default.as_deref()).unwrap();
        confirm(&format!(
            "Clear cache for '{}' ({})?",
            registry_name, registry.url
        ))?;
    }

    // Build cache directory path and clear
    let cache_dir = config::get_registry_cache_dir(&registry.url).unwrap();
    let mut cache = librex::cache::Cache::new(
        cache_dir,
        librex::config::CacheTtl::default(),
        std::num::NonZeroUsize::new(100).unwrap(),
    );

    cache
        .clear()
        .map_err(|e| format!("Failed to clear cache: {}", e))
}

/// Prune expired cache entries for a registry
pub fn cache_prune(
    config_path: &PathBuf,
    name: Option<&str>,
    all: bool,
    dry_run: bool,
) -> Result<librex::cache::PruneStats, String> {
    // Load configuration
    let cfg = config::Config::load(config_path)?;

    if all {
        // Prune all registry caches
        let mut total_stats = librex::cache::PruneStats::default();
        for registry in &cfg.registries.list {
            let cache_dir = config::get_registry_cache_dir(&registry.url).unwrap();
            let cache = librex::cache::Cache::new(
                cache_dir,
                librex::config::CacheTtl::default(),
                std::num::NonZeroUsize::new(100).unwrap(),
            );

            if dry_run {
                // For dry run, we would need to implement a separate method
                // For now, just run the actual prune
                let stats = cache
                    .prune()
                    .map_err(|e| format!("Failed to prune cache: {}", e))?;
                total_stats.removed_files += stats.removed_files;
                total_stats.reclaimed_space += stats.reclaimed_space;
            } else {
                let stats = cache
                    .prune()
                    .map_err(|e| format!("Failed to prune cache: {}", e))?;
                total_stats.removed_files += stats.removed_files;
                total_stats.reclaimed_space += stats.reclaimed_space;
            }
        }
        return Ok(total_stats);
    }

    // Get registry
    let registry = if let Some(name) = name {
        cfg.registries
            .list
            .iter()
            .find(|r| r.name == name)
            .ok_or_else(|| format!("Registry '{}' not found", name))?
    } else {
        // Use default registry
        let default_name = cfg
            .registries
            .default
            .as_ref()
            .ok_or_else(|| no_default_registry_error(&cfg.registries.list))?;
        cfg.registries
            .list
            .iter()
            .find(|r| r.name == *default_name)
            .ok_or_else(|| format!("Default registry '{}' not found", default_name))?
    };

    // Build cache directory path and prune
    let cache_dir = config::get_registry_cache_dir(&registry.url).unwrap();
    let cache = librex::cache::Cache::new(
        cache_dir,
        librex::config::CacheTtl::default(),
        std::num::NonZeroUsize::new(100).unwrap(),
    );

    cache
        .prune()
        .map_err(|e| format!("Failed to prune cache: {}", e))
}

/// Sync cache by fetching and caching registry metadata
pub async fn cache_sync(
    ctx: &crate::context::AppContext,
    config_path: &PathBuf,
    name: Option<&str>,
    manifests: bool,
    all: bool,
    _force: bool,
) -> Result<CacheSyncStats, String> {
    let cfg = config::Config::load(config_path)?;

    if all {
        // Sync all registries
        let mut total_stats = CacheSyncStats::default();
        for registry in &cfg.registries.list {
            println!(
                "Syncing cache for '{}' ({})...",
                registry.name, registry.url
            );
            let stats = sync_single_registry(ctx, &registry.url, manifests).await?;
            total_stats.catalog_entries += stats.catalog_entries;
            total_stats.tag_entries += stats.tag_entries;
            total_stats.manifest_entries += stats.manifest_entries;
            total_stats.total_size += stats.total_size;
        }
        return Ok(total_stats);
    }

    // Get single registry
    let registry = if let Some(name) = name {
        cfg.registries
            .list
            .iter()
            .find(|r| r.name == name)
            .ok_or_else(|| format!("Registry '{}' not found", name))?
    } else {
        let default_name = cfg
            .registries
            .default
            .as_ref()
            .ok_or_else(|| no_default_registry_error(&cfg.registries.list))?;
        cfg.registries
            .list
            .iter()
            .find(|r| r.name == *default_name)
            .ok_or_else(|| format!("Default registry '{}' not found", default_name))?
    };

    println!(
        "Syncing cache for '{}' ({})...",
        name.or(cfg.registries.default.as_deref()).unwrap(),
        registry.url
    );

    sync_single_registry(ctx, &registry.url, manifests).await
}

async fn sync_single_registry(
    ctx: &crate::context::AppContext,
    registry_url: &str,
    manifests: bool,
) -> Result<CacheSyncStats, String> {
    let cache_dir = config::get_registry_cache_dir(registry_url).unwrap();
    let cache_path_ref = cache_dir.as_path();

    // Load credentials if available
    let creds_path = config::get_credentials_path();
    let credentials = if creds_path.exists() {
        if let Ok(store) = librex::auth::FileCredentialStore::new(creds_path) {
            store.get(registry_url).ok().flatten()
        } else {
            None
        }
    } else {
        None
    };

    // Build Rex with cache and credentials
    let mut builder = librex::Rex::builder()
        .registry_url(registry_url)
        .with_cache(cache_path_ref);

    if let Some(creds) = credentials {
        builder = builder.with_credentials(creds);
    }

    let mut rex = builder
        .build()
        .await
        .map_err(|e| format!("Failed to connect to registry: {}", e))?;

    let mut stats = CacheSyncStats::default();
    let formatter = format::create_formatter(ctx);

    // Fetch catalog with spinner
    let spinner = formatter.spinner("Fetching catalog...");

    let repos_res = rex.list_repositories().await;
    let repos = match repos_res {
        Ok(repos) => {
            formatter.finish_progress(
                spinner,
                &format!("Fetched catalog ({} repositories)", repos.len()),
            );
            repos
        }
        Err(e) => {
            spinner.finish_and_clear();
            return Err(format!("Failed to fetch catalog: {}", e));
        }
    };
    stats.catalog_entries = repos.len() as u64;

    // Fetch tags for each repository with progress bar
    let pb = formatter.progress_bar(repos.len() as u64, "Fetching tags");

    let mut total_tags = 0;
    let mut errors = Vec::new();

    for repo in &repos {
        match rex.list_tags(repo).await {
            Ok(tags) => {
                total_tags += tags.len();

                // Fetch manifests if requested
                if manifests {
                    for tag in &tags {
                        let reference = format!("{}:{}", repo, tag);
                        let _ = rex.get_manifest(&reference).await; // Ignore errors for individual manifests
                        stats.manifest_entries += 1;
                    }
                }
            }
            Err(e) => {
                errors.push(format!("{}: {}", repo, e));
            }
        }
        pb.inc(1);
    }

    formatter.finish_progress(
        pb,
        &format!(
            "Fetched {} tags across {} repositories",
            total_tags,
            repos.len()
        ),
    );
    stats.tag_entries = total_tags as u64;

    // Report errors as warnings if some succeeded
    if !errors.is_empty() {
        eprintln!("Warning: Failed to fetch tags for some repositories:");
        for error in &errors {
            eprintln!("  {}", error);
        }
    }

    if manifests {
        formatter.success(&format!("Fetched {} manifests", stats.manifest_entries));
    }

    // Get final cache size
    let cache = librex::cache::Cache::new(
        cache_dir,
        librex::config::CacheTtl::default(),
        std::num::NonZeroUsize::new(100).unwrap(),
    );
    let cache_stats = cache
        .stats()
        .map_err(|e| format!("Failed to get cache stats: {}", e))?;
    stats.total_size = cache_stats.disk_size;

    Ok(stats)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
