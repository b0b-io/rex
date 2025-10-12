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
