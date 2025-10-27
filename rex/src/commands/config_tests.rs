use super::*;

// Note: Most of the logic is tested in config module tests.
// These tests focus on the command handlers and Formattable implementation.

#[test]
fn test_config_format_pretty() {
    let config = config::Config::default();
    let output = config.format_pretty();
    assert!(output.contains("[style]"));
    assert!(output.contains("format = \"pretty\""));
    assert!(output.contains("color = auto"));
    assert!(output.contains("[registries]"));
}

#[test]
fn test_config_format_pretty_with_registries() {
    let mut config = config::Config::default();
    config.registries.default = Some("local".to_string());
    config.registries.list.push(config::RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });

    let output = config.format_pretty();
    assert!(output.contains("default = \"local\""));
    assert!(output.contains("[[registries.list]]"));
    assert!(output.contains("name = \"local\""));
    assert!(output.contains("url = \"http://localhost:5000\""));
}

#[test]
fn test_config_serialization() {
    let config = config::Config::default();
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("style"));
    assert!(json.contains("registries"));
}
