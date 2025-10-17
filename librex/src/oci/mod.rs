//! OCI specification data structures.
//!
//! This module re-exports the necessary data structures from the `oci-spec`
//! crate to provide a single, consistent source for OCI types within `librex`.

pub use oci_spec::image::{Descriptor, ImageConfiguration, ImageIndex, ImageManifest, Platform};

use crate::error::{Result, RexError};

/// Represents either a single-platform image manifest or a multi-platform image index.
///
/// When fetching a manifest from a registry, it may return either:
/// - An `ImageManifest` for single-platform images
/// - An `ImageIndex` for multi-platform images (e.g., linux/amd64, linux/arm64)
///
/// This enum allows handling both cases uniformly.
///
/// # Examples
///
/// ```no_run
/// use librex::oci::ManifestOrIndex;
///
/// # fn example(manifest_or_index: ManifestOrIndex) {
/// match manifest_or_index {
///     ManifestOrIndex::Manifest(manifest) => {
///         println!("Single-platform image with {} layers", manifest.layers().len());
///     }
///     ManifestOrIndex::Index(index) => {
///         println!("Multi-platform image with {} platforms", index.manifests().len());
///         for manifest in index.manifests() {
///             if let Some(platform) = manifest.platform() {
///                 println!("  - {}/{}", platform.os(), platform.architecture());
///             }
///         }
///     }
/// }
/// # }
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ManifestOrIndex {
    /// A single-platform image manifest
    Manifest(ImageManifest),
    /// A multi-platform image index
    Index(ImageIndex),
}

impl ManifestOrIndex {
    /// Parse manifest bytes, automatically detecting whether it's a Manifest or Index.
    ///
    /// This method inspects the JSON to determine the schema version and media type,
    /// then deserializes accordingly.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // Try to parse as a generic JSON value first to inspect the media type
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|e| RexError::validation_with_source("Failed to parse manifest JSON", e))?;

        // Check the mediaType or schemaVersion to determine what we have
        let media_type = value
            .get("mediaType")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Try to determine if it's an index or manifest
        if media_type.contains("index") || media_type.contains("list") {
            // It's an image index (multi-platform)
            let index: ImageIndex = serde_json::from_slice(bytes)
                .map_err(|e| RexError::validation_with_source("Failed to parse image index", e))?;
            Ok(ManifestOrIndex::Index(index))
        } else if media_type.contains("manifest") {
            // It's a single manifest
            let manifest: ImageManifest = serde_json::from_slice(bytes).map_err(|e| {
                RexError::validation_with_source("Failed to parse image manifest", e)
            })?;
            Ok(ManifestOrIndex::Manifest(manifest))
        } else {
            // No mediaType field, try to infer from structure
            // If it has "manifests" array, it's likely an index
            if value.get("manifests").is_some() {
                let index: ImageIndex = serde_json::from_slice(bytes).map_err(|e| {
                    RexError::validation_with_source("Failed to parse image index", e)
                })?;
                Ok(ManifestOrIndex::Index(index))
            } else if value.get("layers").is_some() || value.get("config").is_some() {
                // Has layers or config, likely a manifest
                let manifest: ImageManifest = serde_json::from_slice(bytes).map_err(|e| {
                    RexError::validation_with_source("Failed to parse image manifest", e)
                })?;
                Ok(ManifestOrIndex::Manifest(manifest))
            } else {
                Err(RexError::validation(
                    "Unable to determine if content is a manifest or index",
                ))
            }
        }
    }

    /// Returns true if this is a single-platform manifest.
    pub fn is_manifest(&self) -> bool {
        matches!(self, ManifestOrIndex::Manifest(_))
    }

    /// Returns true if this is a multi-platform index.
    pub fn is_index(&self) -> bool {
        matches!(self, ManifestOrIndex::Index(_))
    }

    /// Returns the manifest if this is a single-platform image.
    pub fn as_manifest(&self) -> Option<&ImageManifest> {
        match self {
            ManifestOrIndex::Manifest(m) => Some(m),
            ManifestOrIndex::Index(_) => None,
        }
    }

    /// Returns the index if this is a multi-platform image.
    pub fn as_index(&self) -> Option<&ImageIndex> {
        match self {
            ManifestOrIndex::Manifest(_) => None,
            ManifestOrIndex::Index(i) => Some(i),
        }
    }

    /// Consumes self and returns the manifest if this is a single-platform image.
    pub fn into_manifest(self) -> Option<ImageManifest> {
        match self {
            ManifestOrIndex::Manifest(m) => Some(m),
            ManifestOrIndex::Index(_) => None,
        }
    }

    /// Consumes self and returns the index if this is a multi-platform image.
    pub fn into_index(self) -> Option<ImageIndex> {
        match self {
            ManifestOrIndex::Manifest(_) => None,
            ManifestOrIndex::Index(i) => Some(i),
        }
    }

    /// Get available platforms if this is an image index.
    ///
    /// Returns a vector of platform descriptors with their corresponding manifests.
    pub fn platforms(&self) -> Vec<(&Platform, &Descriptor)> {
        match self {
            ManifestOrIndex::Manifest(_) => vec![],
            ManifestOrIndex::Index(index) => index
                .manifests()
                .iter()
                .filter_map(|desc| desc.platform().as_ref().map(|platform| (platform, desc)))
                .collect(),
        }
    }

    /// Find a manifest descriptor for a specific platform.
    ///
    /// # Arguments
    ///
    /// * `os` - Operating system (e.g., "linux", "windows")
    /// * `arch` - Architecture (e.g., "amd64", "arm64")
    ///
    /// # Returns
    ///
    /// The descriptor for the matching platform, if found.
    pub fn find_platform(&self, os: &str, arch: &str) -> Option<&Descriptor> {
        match self {
            ManifestOrIndex::Manifest(_) => None,
            ManifestOrIndex::Index(index) => index.manifests().iter().find(|desc| {
                desc.platform().as_ref().is_some_and(|p| {
                    p.os().to_string() == os && p.architecture().to_string() == arch
                })
            }),
        }
    }
}

#[cfg(test)]
mod tests;
