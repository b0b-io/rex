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
