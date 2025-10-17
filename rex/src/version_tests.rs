use super::*;

#[test]
fn test_print_version_contains_package_name() {
    let output = get_version_string();
    assert!(output.contains("rex"));
}

#[test]
fn test_print_version_contains_version_number() {
    let output = get_version_string();
    assert!(output.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_print_version_contains_librex_version() {
    let output = get_version_string();
    assert!(output.contains("librex"));
}

#[test]
fn test_version_string_not_empty() {
    let output = get_version_string();
    assert!(!output.is_empty());
}
