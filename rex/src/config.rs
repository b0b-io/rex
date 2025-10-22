use crate::output::{Formattable, OutputFormat};
use librex::auth::CredentialStore;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tabled::Tabled;
use url::Url;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Style configuration
    #[serde(default)]
    pub style: StyleConfig,
    /// Registry configuration
    #[serde(default)]
    pub registries: RegistriesConfig,
}

/// Style configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    /// Output format: pretty, json, yaml
    #[serde(default = "default_format")]
    pub format: OutputFormat,
    /// Enable color output
    #[serde(default = "default_color")]
    pub color: bool,
}

fn default_format() -> OutputFormat {
    OutputFormat::Pretty
}

fn default_color() -> bool {
    true
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Pretty,
            color: true,
        }
    }
}

/// Registries configuration section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistriesConfig {
    /// Default registry name
    pub default: Option<String>,
    /// List of configured registries
    #[serde(default)]
    pub list: Vec<RegistryEntry>,
}

/// A single registry entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Tabled)]
pub struct RegistryEntry {
    /// Registry name
    #[tabled(rename = "NAME")]
    pub name: String,
    /// Registry URL
    #[tabled(rename = "URL")]
    pub url: String,
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

impl Config {
    /// Load configuration from a file
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let contents =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

        toml::from_str(&contents).map_err(|e| format!("Failed to parse config file: {}", e))
    }

    /// Save configuration to a file
    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(path, toml_str).map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }
}

impl Formattable for Config {
    fn format_pretty(&self) -> String {
        let mut output = format!(
            "[style]\nformat = \"{}\"\ncolor = {}\n",
            match self.style.format {
                OutputFormat::Pretty => "pretty",
                OutputFormat::Json => "json",
                OutputFormat::Yaml => "yaml",
            },
            self.style.color
        );

        // Add registries section
        output.push_str("\n[registries]\n");
        if let Some(ref default) = self.registries.default {
            output.push_str(&format!("default = \"{}\"\n", default));
        }

        if !self.registries.list.is_empty() {
            output.push('\n');
            for registry in &self.registries.list {
                output.push_str(&format!(
                    "[[registries.list]]\nname = \"{}\"\nurl = \"{}\"\n\n",
                    registry.name, registry.url
                ));
            }
        }

        output
    }
}

/// Get the config file path, respecting REX_CONFIG environment variable
pub fn get_config_path() -> PathBuf {
    if let Ok(config_path) = env::var("REX_CONFIG") {
        return PathBuf::from(config_path);
    }

    // Default to ~/.config/rex/config.toml
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("rex").join("config.toml")
    } else {
        // Fallback to current directory
        PathBuf::from("config.toml")
    }
}

/// Initialize a new config file with default values
pub fn init_config(config_path: &PathBuf) -> Result<(), String> {
    if config_path.exists() {
        return Err(
            "Config file already exists. Use 'rex config set' to edit or 'rm' to recreate."
                .to_string(),
        );
    }

    let config = Config::default();
    config.save(config_path)?;

    Ok(())
}

/// Get a configuration value by key (supports nested keys like "style.format")
pub fn get_config_value(config_path: &PathBuf, key: &str) -> Result<String, String> {
    let config = Config::load(config_path)?;

    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["style", "format"] => Ok(match config.style.format {
            OutputFormat::Pretty => "pretty".to_string(),
            OutputFormat::Json => "json".to_string(),
            OutputFormat::Yaml => "yaml".to_string(),
        }),
        ["style", "color"] => Ok(config.style.color.to_string()),
        _ => Err(format!("Unknown config key: {}", key)),
    }
}

/// Set a configuration value by key (supports nested keys like "style.format")
pub fn set_config_value(config_path: &PathBuf, key: &str, value: &str) -> Result<(), String> {
    // Load existing config or create default
    let mut config = if config_path.exists() {
        Config::load(config_path)?
    } else {
        Config::default()
    };

    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["style", "format"] => {
            config.style.format = OutputFormat::from(value);
        }
        ["style", "color"] => {
            config.style.color = value.parse::<bool>().map_err(|_| {
                format!("Invalid boolean value '{}'. Use 'true' or 'false'.", value)
            })?;
        }
        _ => return Err(format!("Unknown config key: {}", key)),
    }

    config.save(config_path)?;

    Ok(())
}

/// Open the config file in the user's editor
pub fn edit_config(config_path: &PathBuf) -> Result<(), String> {
    // Get editor from environment
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .map_err(|_| "No editor found. Set EDITOR or VISUAL environment variable.".to_string())?;

    // Create config file if it doesn't exist
    if !config_path.exists() {
        let config = Config::default();
        config.save(config_path)?;
    }

    // Open editor
    let status = Command::new(editor)
        .arg(config_path)
        .status()
        .map_err(|e| format!("Failed to open editor: {}", e))?;

    if !status.success() {
        return Err("Editor exited with non-zero status".to_string());
    }

    Ok(())
}

/// Display the entire configuration
pub fn display_config(config_path: &PathBuf) -> Result<Config, String> {
    Config::load(config_path)
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
/// # use std::path::PathBuf;
/// # use rex::config::validate_registry_url;
/// assert!(validate_registry_url("https://registry-1.docker.io").is_ok());
/// assert!(validate_registry_url("localhost:5000").is_ok());
/// assert!(validate_registry_url("http:://bad").is_err());
/// ```
pub fn validate_registry_url(url_str: &str) -> Result<String, String> {
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
pub fn init_registry(config_path: &PathBuf, name: &str, url: &str) -> Result<(), String> {
    // Load existing config or create default
    let mut config = if config_path.exists() {
        Config::load(config_path)?
    } else {
        Config::default()
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
pub fn remove_registry(config_path: &PathBuf, name: &str) -> Result<(), String> {
    // Load existing config
    let mut config = Config::load(config_path)?;

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
pub fn use_registry(config_path: &PathBuf, name: &str) -> Result<(), String> {
    // Load existing config
    let mut config = Config::load(config_path)?;

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
pub fn show_registry(config_path: &PathBuf, name: &str) -> Result<RegistryDisplay, String> {
    // Load existing config
    let config = Config::load(config_path)?;

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

/// Handle the config init subcommand
pub fn handle_init() {
    let config_path = get_config_path();
    match init_config(&config_path) {
        Ok(_) => {
            println!("Initialized config file at: {}", config_path.display());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the config get subcommand
pub fn handle_get(key: Option<&str>, format: OutputFormat) {
    let config_path = get_config_path();

    match key {
        Some(k) => {
            // Get specific key
            match get_config_value(&config_path, k) {
                Ok(value) => println!("{}", value),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            // Display all config
            match display_config(&config_path) {
                Ok(config) => match crate::output::format_output(&config, format) {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        eprintln!("Error formatting output: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Handle the config set subcommand
pub fn handle_set(key: Option<&str>, value: Option<&str>) {
    let config_path = get_config_path();

    match (key, value) {
        (Some(k), Some(v)) => {
            // Set specific key
            match set_config_value(&config_path, k, v) {
                Ok(_) => println!("Set {} = {}", k, v),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        (None, None) => {
            // Open editor
            match edit_config(&config_path) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!(
                "Error: Invalid arguments. Use 'rex config set <key> <value>' or 'rex config set' to edit."
            );
            std::process::exit(1);
        }
    }
}

/// List all registries
pub fn list_registries(config_path: &PathBuf) -> Result<Vec<RegistryDisplay>, String> {
    let config = Config::load(config_path)?;

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

/// Handle the registry init subcommand
pub fn handle_registry_init(name: &str, url: &str) {
    let config_path = get_config_path();
    match init_registry(&config_path, name, url) {
        Ok(_) => println!("Initialized registry '{}' at {}", name, url),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry remove subcommand
pub fn handle_registry_remove(name: &str) {
    let config_path = get_config_path();
    match remove_registry(&config_path, name) {
        Ok(_) => println!("Removed registry '{}'", name),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry use subcommand
pub fn handle_registry_use(name: &str) {
    let config_path = get_config_path();
    match use_registry(&config_path, name) {
        Ok(_) => println!("Set '{}' as default registry", name),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry show subcommand
pub fn handle_registry_show(name: &str, format: OutputFormat) {
    let config_path = get_config_path();
    match show_registry(&config_path, name) {
        Ok(registry) => match crate::output::format_output(&registry, format) {
            Ok(output) => println!("{}", output),
            Err(e) => {
                eprintln!("Error formatting output: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry list subcommand
pub fn handle_registry_list(format: OutputFormat) {
    let config_path = get_config_path();
    match list_registries(&config_path) {
        Ok(registries) => {
            if registries.is_empty() {
                println!("No registries configured.");
                return;
            }

            match format {
                OutputFormat::Pretty => {
                    use tabled::Table;
                    let table = Table::new(&registries).to_string();
                    println!("{}", table);
                }
                OutputFormat::Json => match serde_json::to_string_pretty(&registries) {
                    Ok(json) => println!("{}", json),
                    Err(e) => {
                        eprintln!("Error formatting JSON: {}", e);
                        std::process::exit(1);
                    }
                },
                OutputFormat::Yaml => match serde_yaml::to_string(&registries) {
                    Ok(yaml) => print!("{}", yaml),
                    Err(e) => {
                        eprintln!("Error formatting YAML: {}", e);
                        std::process::exit(1);
                    }
                },
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Check registry connectivity and status
pub(crate) async fn check_registry(config_path: &PathBuf, name: &str) -> RegistryCheckResult {
    // Load config
    let config = match Config::load(config_path) {
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
    let creds_path = get_credentials_path();
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

/// Handle the registry check subcommand
pub async fn handle_registry_check(name: &str, format: OutputFormat) {
    let config_path = get_config_path();
    let result = check_registry(&config_path, name).await;

    match crate::output::format_output(&result, format) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("Error formatting output: {}", e);
            std::process::exit(1);
        }
    }
}

/// Get credentials path
pub fn get_credentials_path() -> PathBuf {
    if let Ok(creds_path) = env::var("REX_CREDENTIALS") {
        return PathBuf::from(creds_path);
    }

    // Default to ~/.config/rex/credentials.toml
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("rex").join("credentials.toml")
    } else {
        // Fallback to current directory
        PathBuf::from("credentials.toml")
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

/// Handle the registry login subcommand
pub async fn handle_registry_login(name: &str, username: Option<&str>, password: Option<&str>) {
    let config_path = get_config_path();

    // Load config to verify registry exists
    let config = match Config::load(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Find registry
    let registry = match config.registries.list.iter().find(|r| r.name == name) {
        Some(reg) => reg,
        None => {
            eprintln!(
                "Error: Registry '{}' not found. Use 'rex registry add' to add it first.",
                name
            );
            std::process::exit(1);
        }
    };

    // Get credentials
    let username = match prompt_username(username) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let password = match prompt_password(password) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Create credentials
    let credentials = librex::auth::Credentials::basic(&username, &password);

    // Verify credentials by attempting to authenticate with the registry
    println!("Verifying credentials...");
    let client = match librex::client::Client::new(&registry.url) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Invalid registry URL: {}", e);
            std::process::exit(1);
        }
    };

    match client
        .check_version_with_credentials(Some(&credentials))
        .await
    {
        Ok(_) => {
            println!("✓ Credentials verified successfully");
        }
        Err(e) => {
            let error_str = format!("{}", e);
            if error_str.contains("Authentication") || error_str.contains("401") {
                eprintln!("Error: Authentication failed. Please check your username and password.");
                std::process::exit(1);
            } else if error_str.contains("403") || error_str.contains("Forbidden") {
                eprintln!(
                    "Error: Access forbidden. Your credentials may not have the required permissions."
                );
                std::process::exit(1);
            } else {
                eprintln!("Error: Failed to verify credentials: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Store credentials
    let creds_path = get_credentials_path();
    let mut store = match librex::auth::FileCredentialStore::new(creds_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to initialize credential store: {}", e);
            std::process::exit(1);
        }
    };

    match store.store(&registry.url, &credentials) {
        Ok(_) => println!("Successfully stored credentials for '{}'", name),
        Err(e) => {
            eprintln!("Error: Failed to store credentials: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry logout subcommand
pub fn handle_registry_logout(name: &str) {
    let config_path = get_config_path();

    // Load config to verify registry exists and get URL
    let config = match Config::load(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Find registry
    let registry = match config.registries.list.iter().find(|r| r.name == name) {
        Some(reg) => reg,
        None => {
            eprintln!(
                "Error: Registry '{}' not found. Use 'rex registry list' to see configured registries.",
                name
            );
            std::process::exit(1);
        }
    };

    // Remove credentials
    let creds_path = get_credentials_path();
    let mut store = match librex::auth::FileCredentialStore::new(creds_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to initialize credential store: {}", e);
            std::process::exit(1);
        }
    };

    match store.remove(&registry.url) {
        Ok(_) => println!("Successfully logged out from '{}'", name),
        Err(e) => {
            eprintln!("Error: Failed to remove credentials: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
