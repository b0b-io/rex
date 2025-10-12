//! OCI specification data structures.
//!
//! This module re-exports the necessary data structures from the `oci-spec`
//! crate to provide a single, consistent source for OCI types within `librex`.

pub use oci_spec::image::{Descriptor, ImageConfiguration, ImageIndex, ImageManifest, Platform};

#[cfg(test)]
mod tests;
