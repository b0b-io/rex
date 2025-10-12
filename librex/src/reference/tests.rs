use super::*;
use std::str::FromStr;

#[test]
fn test_reference_from_valid_string_succeeds() {
    let valid_ref = "ghcr.io/user/repo:latest";
    let reference = Reference::from_str(valid_ref);
    assert!(reference.is_ok());
}

#[test]
fn test_reference_from_invalid_string_fails() {
    let invalid_ref = "Invalid-Reference-With-Caps";
    let reference = Reference::from_str(invalid_ref);
    assert!(reference.is_err());
    assert!(matches!(
        reference.unwrap_err(),
        RexError::Validation { .. }
    ));
}

#[test]
fn test_reference_display_trait() {
    let valid_ref = "ghcr.io/user/repo:latest";
    let reference = Reference::from_str(valid_ref).unwrap();
    assert_eq!(reference.to_string(), valid_ref);
}

#[test]
fn test_reference_accessors() {
    let valid_ref = "ghcr.io/user/repo:latest";
    let reference = Reference::from_str(valid_ref).unwrap();
    assert_eq!(reference.registry(), "ghcr.io");
    assert_eq!(reference.repository(), "user/repo");
    assert_eq!(reference.tag(), Some("latest"));
    assert_eq!(reference.digest(), None);
}

#[test]
fn test_reference_with_digest() {
    let valid_ref =
        "ghcr.io/user/repo@sha256:7173b809ca12ec5dee4506cd86be934c4596dd234ee82c0662eac04a8c2c71dc";
    let reference = Reference::from_str(valid_ref).unwrap();
    assert_eq!(reference.registry(), "ghcr.io");
    assert_eq!(reference.repository(), "user/repo");
    assert_eq!(reference.tag(), None);
    assert!(reference.digest().is_some());
}

#[test]
fn test_reference_with_nested_repository_path() {
    let valid_ref = "ghcr.io/org/team/project/app:v1.0.0";
    let reference = Reference::from_str(valid_ref).unwrap();
    assert_eq!(reference.registry(), "ghcr.io");
    assert_eq!(reference.repository(), "org/team/project/app");
    assert_eq!(reference.tag(), Some("v1.0.0"));
}

#[test]
fn test_reference_with_port() {
    let valid_ref = "localhost:5000/myrepo:latest";
    let reference = Reference::from_str(valid_ref).unwrap();
    assert_eq!(reference.registry(), "localhost:5000");
    assert_eq!(reference.repository(), "myrepo");
    assert_eq!(reference.tag(), Some("latest"));
}

#[test]
fn test_reference_simple_name_only() {
    let valid_ref = "alpine";
    let reference = Reference::from_str(valid_ref).unwrap();
    // oci-spec adds "library/" prefix for simple names (Docker Hub convention)
    assert_eq!(reference.repository(), "library/alpine");
}

#[test]
fn test_reference_with_tag_no_registry() {
    let valid_ref = "alpine:3.19";
    let reference = Reference::from_str(valid_ref).unwrap();
    // oci-spec adds "library/" prefix for simple names (Docker Hub convention)
    assert_eq!(reference.repository(), "library/alpine");
    assert_eq!(reference.tag(), Some("3.19"));
}

#[test]
fn test_reference_semver_tag() {
    let valid_ref = "ghcr.io/user/repo:v1.2.3-alpha.1";
    let reference = Reference::from_str(valid_ref).unwrap();
    assert_eq!(reference.registry(), "ghcr.io");
    assert_eq!(reference.repository(), "user/repo");
    assert_eq!(reference.tag(), Some("v1.2.3-alpha.1"));
}

#[test]
fn test_reference_inner_accessor() {
    let valid_ref = "ghcr.io/user/repo:latest";
    let reference = Reference::from_str(valid_ref).unwrap();
    assert_eq!(reference.inner().to_string(), valid_ref);
}
