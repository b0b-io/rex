use super::*;
use std::str::FromStr;

#[test]
fn test_digest_from_valid_string_succeeds() {
    let valid_digest_str =
        "sha256:7173b809ca12ec5dee4506cd86be934c4596dd234ee82c0662eac04a8c2c71dc";
    let digest = Digest::from_str(valid_digest_str);
    assert!(digest.is_ok());
}

#[test]
fn test_digest_from_invalid_string_fails() {
    let invalid_digest_str = "sha256:invalid-digest";
    let digest = Digest::from_str(invalid_digest_str);
    assert!(digest.is_err());
    assert!(matches!(digest.unwrap_err(), RexError::Validation { .. }));
}

#[test]
fn test_digest_display_trait() {
    let digest_str = "sha256:7173b809ca12ec5dee4506cd86be934c4596dd234ee82c0662eac04a8c2c71dc";
    let digest = Digest::from_str(digest_str).unwrap();
    assert_eq!(digest.to_string(), digest_str);
}
