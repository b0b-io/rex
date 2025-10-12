//! OCI Content Digest validation and manipulation.
//!
//! This module provides a wrapper around the `oci_spec::image::Digest` type
//! to integrate with Rex's error handling and provide a consistent API.

use crate::error::{Result, RexError};
use oci_spec::image::Digest as OciDigest;
use std::fmt;
use std::str::FromStr;

#[cfg(test)]
mod tests;

/// Represents a content digest, wrapping the `oci_spec::image::Digest` type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest(OciDigest);

impl FromStr for Digest {
    type Err = RexError;

    fn from_str(s: &str) -> Result<Self> {
        let oci_digest = OciDigest::from_str(s).map_err(|e| RexError::Validation {
            message: format!("Invalid digest format: {}", e),
            source: Some(Box::new(e)),
        })?;
        Ok(Digest(oci_digest))
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Digest {
    /// Returns the algorithm part of the digest as a string (e.g., "sha256").
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::digest::Digest;
    /// use std::str::FromStr;
    ///
    /// let digest = Digest::from_str("sha256:7173b809ca12ec5dee4506cd86be934c4596dd234ee82c0662eac04a8c2c71dc").unwrap();
    /// assert_eq!(digest.algorithm(), "sha256");
    /// ```
    pub fn algorithm(&self) -> String {
        self.0.algorithm().to_string()
    }

    /// Returns the hex-encoded hash part of the digest.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::digest::Digest;
    /// use std::str::FromStr;
    ///
    /// let digest = Digest::from_str("sha256:7173b809ca12ec5dee4506cd86be934c4596dd234ee82c0662eac04a8c2c71dc").unwrap();
    /// assert_eq!(digest.hex(), "7173b809ca12ec5dee4506cd86be934c4596dd234ee82c0662eac04a8c2c71dc");
    /// ```
    pub fn hex(&self) -> &str {
        self.0.digest()
    }

    /// Returns a reference to the underlying `oci_spec::image::Digest`.
    pub fn inner(&self) -> &OciDigest {
        &self.0
    }
}
