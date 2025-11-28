//! OCI Image Reference parsing and manipulation.
//!
//! This module provides a wrapper around the `oci_spec::image::Reference`
//! type to integrate with Rex's error handling and provide a consistent API.

use crate::error::{Result, RexError};
use oci_spec::distribution::Reference as OciReference;
use std::fmt;
use std::str::FromStr;

#[cfg(test)]
mod tests;

/// Represents an OCI image reference, wrapping `oci_spec::distribution::Reference`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference(OciReference);

impl FromStr for Reference {
    type Err = RexError;

    fn from_str(s: &str) -> Result<Self> {
        let oci_reference = OciReference::from_str(s).map_err(|e| RexError::Validation {
            message: format!("Invalid image reference: {}", e),
            source: Some(Box::new(e)),
        })?;
        Ok(Reference(oci_reference))
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Reference {
    /// Returns the registry part of the reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::reference::Reference;
    /// use std::str::FromStr;
    ///
    /// let reference = Reference::from_str("ghcr.io/user/repo:latest").unwrap();
    /// assert_eq!(reference.registry(), "ghcr.io");
    /// ```
    pub fn registry(&self) -> &str {
        self.0.registry()
    }

    /// Returns the repository part of the reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::reference::Reference;
    /// use std::str::FromStr;
    ///
    /// let reference = Reference::from_str("ghcr.io/user/repo:latest").unwrap();
    /// assert_eq!(reference.repository(), "user/repo");
    /// ```
    pub fn repository(&self) -> &str {
        self.0.repository()
    }

    /// Returns the repository part, optionally stripping auto-added "library/" prefix.
    ///
    /// The oci-spec library automatically adds "library/" prefix for simple repository
    /// names (e.g., "golang" → "library/golang") following Docker Hub convention.
    ///
    /// When `dockerhub_compat` is false, this method strips the "library/" prefix
    /// ONLY if it was auto-added (i.e., the rest has no slashes). User-provided
    /// "library/" prefixes in paths like "library/myrepo/subpath" are preserved.
    ///
    /// # Arguments
    ///
    /// * `dockerhub_compat` - If true, keeps "library/" prefix; if false, strips auto-added prefix
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::reference::Reference;
    /// use std::str::FromStr;
    ///
    /// // Simple name: "golang" gets parsed as "library/golang" by oci-spec
    /// let ref1 = Reference::from_str("golang:latest").unwrap();
    /// assert_eq!(ref1.repository_for_registry(false), "golang");
    /// assert_eq!(ref1.repository_for_registry(true), "library/golang");
    ///
    /// // Note: "library/myrepo" without additional slashes is indistinguishable
    /// // from auto-added prefix, so it gets stripped when dockerhub_compat=false
    /// let ref2 = Reference::from_str("library/myrepo:latest").unwrap();
    /// assert_eq!(ref2.repository_for_registry(false), "myrepo");
    ///
    /// // Organization repo: no prefix added
    /// let ref3 = Reference::from_str("myorg/repo:latest").unwrap();
    /// assert_eq!(ref3.repository_for_registry(false), "myorg/repo");
    /// ```
    pub fn repository_for_registry(&self, dockerhub_compat: bool) -> &str {
        let repo = self.0.repository();

        if !dockerhub_compat && repo.starts_with("library/") {
            let after_prefix = &repo[8..]; // Everything after "library/"

            // Only strip if it was auto-added (i.e., simple name with no more slashes)
            // "library/golang" → strip (auto-added for simple "golang")
            // "library/myrepo/sub" → keep (user explicitly provided "library/myrepo/sub")
            if !after_prefix.contains('/') {
                return after_prefix;
            }
        }

        repo
    }

    /// Returns the tag part of the reference, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::reference::Reference;
    /// use std::str::FromStr;
    ///
    /// let reference = Reference::from_str("ghcr.io/user/repo:latest").unwrap();
    /// assert_eq!(reference.tag(), Some("latest"));
    /// ```
    pub fn tag(&self) -> Option<&str> {
        self.0.tag()
    }

    /// Returns the digest part of the reference, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use librex::reference::Reference;
    /// use std::str::FromStr;
    ///
    /// let reference = Reference::from_str("ghcr.io/user/repo@sha256:7173b809ca12ec5dee4506cd86be934c4596dd234ee82c0662eac04a8c2c71dc").unwrap();
    /// assert!(reference.digest().is_some());
    /// ```
    pub fn digest(&self) -> Option<&str> {
        self.0.digest()
    }

    /// Returns a reference to the underlying `oci_spec::distribution::Reference`.
    pub fn inner(&self) -> &OciReference {
        &self.0
    }
}
