use super::*;

#[test]
fn test_rex_builder_new() {
    let builder = RexBuilder::new("http://localhost:5000".to_string());
    assert_eq!(builder.registry_url, "http://localhost:5000");
    assert!(builder.config_path.is_none());
}
