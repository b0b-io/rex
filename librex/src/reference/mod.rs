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
    pub fn registry(&self) -> &str {
        self.0.registry()
    }

    /// Returns the repository part of the reference.
    pub fn repository(&self) -> &str {
        self.0.repository()
    }

    /// Returns the tag part of the reference, if present.
    pub fn tag(&self) -> Option<&str> {
        self.0.tag()
    }

    /// Returns the digest part of the reference, if present.
    pub fn digest(&self) -> Option<&str> {
        self.0.digest()
    }
}
