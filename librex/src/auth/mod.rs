//! Authentication handling for OCI registries.
//!
//! This module provides authentication support for OCI-compliant registries,
//! including anonymous access, Basic authentication, and Bearer token authentication
//! following the OCI Distribution Specification authentication flow.

use crate::error::{Result, RexError};

#[cfg(test)]
mod tests;

/// Credentials for registry authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credentials {
    /// No authentication required (anonymous access)
    Anonymous,

    /// HTTP Basic authentication with username and password
    Basic {
        /// Username for authentication
        username: String,
        /// Password for authentication
        password: String,
    },

    /// Bearer token authentication (OAuth2-style)
    Bearer {
        /// The bearer token
        token: String,
    },
}

impl Credentials {
    /// Creates anonymous credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::auth::Credentials;
    ///
    /// let creds = Credentials::anonymous();
    /// ```
    pub fn anonymous() -> Self {
        Self::Anonymous
    }

    /// Creates Basic authentication credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::auth::Credentials;
    ///
    /// let creds = Credentials::basic("username", "password");
    /// ```
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Creates Bearer token credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::auth::Credentials;
    ///
    /// let creds = Credentials::bearer("token123");
    /// ```
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer {
            token: token.into(),
        }
    }

    /// Returns the Authorization header value for these credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::auth::Credentials;
    ///
    /// let creds = Credentials::basic("user", "pass");
    /// let header = creds.to_header_value();
    /// assert!(header.is_some());
    /// ```
    pub fn to_header_value(&self) -> Option<String> {
        match self {
            Self::Anonymous => None,
            Self::Basic { username, password } => {
                use base64::{Engine as _, engine::general_purpose};
                let credentials = format!("{}:{}", username, password);
                let encoded = general_purpose::STANDARD.encode(credentials);
                Some(format!("Basic {}", encoded))
            }
            Self::Bearer { token } => Some(format!("Bearer {}", token)),
        }
    }
}

/// Information parsed from a WWW-Authenticate header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthChallenge {
    /// The authentication scheme (e.g., "Bearer")
    pub scheme: String,

    /// The authentication realm
    pub realm: String,

    /// The service identifier
    pub service: Option<String>,

    /// The scope being requested
    pub scope: Option<String>,
}

impl AuthChallenge {
    /// Parses a WWW-Authenticate header value.
    ///
    /// Example header: `Bearer realm="https://auth.example.com/token",service="registry.example.com",scope="repository:alpine:pull"`
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::auth::AuthChallenge;
    ///
    /// let header = r#"Bearer realm="https://auth.example.com/token",service="registry""#;
    /// let challenge = AuthChallenge::parse(header).unwrap();
    /// assert_eq!(challenge.scheme, "Bearer");
    /// ```
    pub fn parse(header: &str) -> Result<Self> {
        let header = header.trim();

        // Split scheme from parameters
        let (scheme, params) = header
            .split_once(' ')
            .ok_or_else(|| RexError::validation("Invalid WWW-Authenticate header format"))?;

        // Parse parameters
        let mut realm = None;
        let mut service = None;
        let mut scope = None;

        for param in params.split(',') {
            let param = param.trim();
            if let Some((key, value)) = param.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match key {
                    "realm" => realm = Some(value.to_string()),
                    "service" => service = Some(value.to_string()),
                    "scope" => scope = Some(value.to_string()),
                    _ => {} // Ignore unknown parameters
                }
            }
        }

        let realm = realm.ok_or_else(|| {
            RexError::validation("WWW-Authenticate header missing required 'realm' parameter")
        })?;

        Ok(Self {
            scheme: scheme.to_string(),
            realm,
            service,
            scope,
        })
    }
}
