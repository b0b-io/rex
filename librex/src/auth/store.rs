//! Credential storage abstraction for registry authentication.
//!
//! This module provides a trait-based abstraction for storing and retrieving
//! registry credentials. The file-based implementation stores credentials in
//! a TOML file with restricted permissions (0600).
//!
//! Future implementations could include OS keyring integration.

use crate::auth::Credentials;
use crate::error::{Result, RexError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Trait for storing and retrieving registry credentials.
///
/// This trait allows for different credential storage backends
/// (file-based, OS keyring, etc.) to be used interchangeably.
pub trait CredentialStore {
    /// Store credentials for a registry.
    ///
    /// # Arguments
    ///
    /// * `registry` - The registry hostname or identifier
    /// * `credentials` - The credentials to store
    ///
    /// # Errors
    ///
    /// Returns an error if the credentials cannot be stored.
    fn store(&mut self, registry: &str, credentials: &Credentials) -> Result<()>;

    /// Retrieve credentials for a registry.
    ///
    /// # Arguments
    ///
    /// * `registry` - The registry hostname or identifier
    ///
    /// # Returns
    ///
    /// Returns `Some(Credentials)` if credentials are found, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the credentials cannot be read.
    fn get(&self, registry: &str) -> Result<Option<Credentials>>;

    /// Remove credentials for a registry.
    ///
    /// # Arguments
    ///
    /// * `registry` - The registry hostname or identifier
    ///
    /// # Errors
    ///
    /// Returns an error if the credentials cannot be removed.
    fn remove(&mut self, registry: &str) -> Result<()>;

    /// List all registries with stored credentials.
    ///
    /// # Returns
    ///
    /// Returns a vector of registry identifiers that have stored credentials.
    ///
    /// # Errors
    ///
    /// Returns an error if the credential list cannot be retrieved.
    fn list(&self) -> Result<Vec<String>>;
}

/// Stored credential representation for serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct StoredCredential {
    /// Username for Basic authentication
    username: String,
    /// Password for Basic authentication (base64 encoded)
    password: String,
}

/// File-based credential store implementation.
///
/// Stores credentials in a TOML file with restricted permissions (0600).
/// Credentials are base64 encoded for basic obfuscation.
///
/// # Security Note
///
/// This is a simple file-based storage with basic encoding. For production
/// use with sensitive credentials, consider using OS keyring integration.
///
/// # Examples
///
/// ```no_run
/// use librex::auth::{Credentials, FileCredentialStore, CredentialStore};
/// use std::path::PathBuf;
///
/// # fn example() -> librex::error::Result<()> {
/// let path = PathBuf::from("/home/user/.config/rex/credentials.toml");
/// let mut store = FileCredentialStore::new(path)?;
///
/// // Store credentials
/// let creds = Credentials::basic("username", "password");
/// store.store("registry.example.com", &creds)?;
///
/// // Retrieve credentials
/// let retrieved = store.get("registry.example.com")?;
/// assert!(retrieved.is_some());
///
/// // Remove credentials
/// store.remove("registry.example.com")?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct FileCredentialStore {
    /// Path to the credentials file
    path: PathBuf,
    /// In-memory cache of credentials
    credentials: HashMap<String, StoredCredential>,
}

impl FileCredentialStore {
    /// Creates a new file-based credential store.
    ///
    /// If the file exists, it will be loaded. If not, an empty store is created.
    /// The parent directory will be created if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the credentials file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory cannot be created
    /// - The file exists but cannot be read or parsed
    /// - File permissions cannot be set
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use librex::auth::FileCredentialStore;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> librex::error::Result<()> {
    /// let path = PathBuf::from("/home/user/.config/rex/credentials.toml");
    /// let store = FileCredentialStore::new(path)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: PathBuf) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                RexError::config_with_source(
                    "Failed to create credentials directory",
                    parent.to_str(),
                    e,
                )
            })?;
        }

        // Load existing credentials or create empty map
        let credentials = if path.exists() {
            Self::load_from_file(&path)?
        } else {
            HashMap::new()
        };

        Ok(Self { path, credentials })
    }

    /// Loads credentials from the file.
    fn load_from_file(path: &PathBuf) -> Result<HashMap<String, StoredCredential>> {
        let contents = fs::read_to_string(path).map_err(|e| {
            RexError::config_with_source("Failed to read credentials file", path.to_str(), e)
        })?;

        toml::from_str(&contents).map_err(|e| {
            RexError::config_with_source("Failed to parse credentials file", path.to_str(), e)
        })
    }

    /// Saves credentials to the file with restricted permissions.
    fn save_to_file(&self) -> Result<()> {
        let contents = toml::to_string_pretty(&self.credentials).map_err(|e| {
            RexError::config_with_source("Failed to serialize credentials", self.path.to_str(), e)
        })?;

        fs::write(&self.path, contents).map_err(|e| {
            RexError::config_with_source("Failed to write credentials file", self.path.to_str(), e)
        })?;

        // Set file permissions to 0600 (user read/write only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.path, permissions).map_err(|e| {
                RexError::config_with_source(
                    "Failed to set credentials file permissions",
                    self.path.to_str(),
                    e,
                )
            })?;
        }

        Ok(())
    }

    /// Encodes a credential for storage.
    fn encode_credential(credentials: &Credentials) -> Result<StoredCredential> {
        match credentials {
            Credentials::Basic { username, password } => {
                use base64::{Engine as _, engine::general_purpose};
                let encoded_password = general_purpose::STANDARD.encode(password);
                Ok(StoredCredential {
                    username: username.clone(),
                    password: encoded_password,
                })
            }
            Credentials::Anonymous => {
                Err(RexError::validation("Cannot store anonymous credentials"))
            }
            Credentials::Bearer { .. } => Err(RexError::validation(
                "Bearer token storage not yet supported",
            )),
        }
    }

    /// Decodes a stored credential.
    fn decode_credential(stored: &StoredCredential) -> Result<Credentials> {
        use base64::{Engine as _, engine::general_purpose};
        let decoded_password = general_purpose::STANDARD
            .decode(&stored.password)
            .map_err(|e| RexError::validation_with_source("Failed to decode password", e))?;

        let password = String::from_utf8(decoded_password)
            .map_err(|e| RexError::validation_with_source("Invalid password encoding", e))?;

        Ok(Credentials::Basic {
            username: stored.username.clone(),
            password,
        })
    }
}

impl CredentialStore for FileCredentialStore {
    fn store(&mut self, registry: &str, credentials: &Credentials) -> Result<()> {
        let stored = Self::encode_credential(credentials)?;
        self.credentials.insert(registry.to_string(), stored);
        self.save_to_file()
    }

    fn get(&self, registry: &str) -> Result<Option<Credentials>> {
        match self.credentials.get(registry) {
            Some(stored) => Ok(Some(Self::decode_credential(stored)?)),
            None => Ok(None),
        }
    }

    fn remove(&mut self, registry: &str) -> Result<()> {
        self.credentials.remove(registry);
        self.save_to_file()
    }

    fn list(&self) -> Result<Vec<String>> {
        Ok(self.credentials.keys().cloned().collect())
    }
}
