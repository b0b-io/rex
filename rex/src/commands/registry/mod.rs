use crate::config::{self, RegistryEntry};
use crate::format::Formattable;
use librex::auth::CredentialStore;
use serde::Serialize;
use std::path::PathBuf;
use tabled::Tabled;
use url::Url;

pub mod handlers;

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
            output.push_str("Status: ✓ Online\n");
            if let Some(ref api_version) = self.api_version {
                output.push_str(&format!("API Version: {}\n", api_version));
            }

            // Show auth status
            if self.authenticated {
                output.push_str("Authentication: ✓ Authenticated\n");
            } else if self.auth_required {
                output.push_str("Authentication: ⚠ Required (not configured)\n");
            } else {
                output.push_str("Authentication: ○ Not required\n");
            }
        } else {
            output.push_str("Status: ✗ Offline\n");
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
pub(crate) fn remove_registry(config_path: &PathBuf, name: &str) -> Result<(), String> {
    // Load existing config
    let mut config = config::Config::load(config_path)?;

    // Find the registry index
    let registry_index = config
        .registries
        .list
        .iter()
        .position(|r| r.name == name)
        .ok_or_else(|| format!("Registry '{}' not found", name))?;

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
pub(crate) async fn check_registry(config_path: &PathBuf, name: &str) -> RegistryCheckResult {
    // Load config
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

    // Create client and check version
    let client = match librex::client::Client::new(&registry.url) {
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
    let client = librex::client::Client::new(&registry.url)
        .map_err(|e| format!("Invalid registry URL: {}", e))?;

    client
        .check_version_with_credentials(Some(&credentials))
        .await
        .map_err(|e| {
            let error_str = format!("{}", e);
            if error_str.contains("Authentication") || error_str.contains("401") {
                "Authentication failed. Please check your username and password.".to_string()
            } else if error_str.contains("403") || error_str.contains("Forbidden") {
                "Access forbidden. Your credentials may not have the required permissions."
                    .to_string()
            } else {
                format!("Failed to verify credentials: {}", e)
            }
        })?;

    println!("✓ Credentials verified successfully");

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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
