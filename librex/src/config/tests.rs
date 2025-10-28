use super::*;

#[test]
fn test_default_config() {
    let config = Config::default();

    // Verify default output settings
    assert_eq!(config.output.format, OutputFormat::Pretty);
    assert_eq!(config.output.color, ColorChoice::Auto);

    // Verify default network settings
    assert_eq!(config.network.timeout, 30);

    // Verify default cache settings
    assert!(config.cache.enabled);
    assert_eq!(config.cache.ttl.catalog, 3600);
    assert_eq!(config.cache.ttl.tags, 1800);
    assert_eq!(config.cache.ttl.manifest, 86400);
    assert_eq!(config.cache.ttl.config, 31536000);
    assert_eq!(config.cache.limits.memory_entries, 1000);

    // Verify default TUI settings
    assert_eq!(config.tui.theme, "dark");
    assert!(config.tui.vim_bindings);

    // Verify default registries settings
    assert!(config.registries.current.is_none());
    assert!(config.registries.list.is_empty());
}

#[test]
fn test_from_str_empty_yaml() {
    let yaml = "";
    let config = Config::from_yaml_str(yaml).unwrap();
    // Should be equivalent to default
    assert_eq!(config, Config::default());
}

#[test]
fn test_from_str_partial_yaml() {
    let yaml = r#"
output:
  format: json
network:
  timeout: 60
registries:
  current: prod
"#;
    let config = Config::from_yaml_str(yaml).unwrap();

    // Check specified values
    assert_eq!(config.output.format, OutputFormat::Json);
    assert_eq!(config.network.timeout, 60);
    assert_eq!(config.registries.current, Some("prod".to_string()));

    // Check that other values are still default
    assert_eq!(config.output.color, ColorChoice::Auto); // Default
    assert!(config.cache.enabled); // Default
}

#[test]
fn test_from_str_full_yaml() {
    let yaml = r#"
output:
  format: yaml
  color: never
network:
  timeout: 10
cache:
  enabled: false
  ttl:
    catalog: 60
    tags: 60
    manifest: 3600
    config: 3600
  limits:
    memory_entries: 100
    disk_entries: 500
tui:
  theme: light
  vim_bindings: false
registries:
  current: local
  list:
    - name: local
      url: "http://localhost:5000"
      insecure: true
    - name: prod
      url: "https://registry.example.com"
"#;
    let config = Config::from_yaml_str(yaml).unwrap();

    assert_eq!(config.output.format, OutputFormat::Yaml);
    assert_eq!(config.output.color, ColorChoice::Never);
    assert_eq!(config.network.timeout, 10);
    assert!(!config.cache.enabled);
    assert_eq!(config.cache.ttl.tags, 60);
    assert_eq!(config.cache.limits.memory_entries, 100);
    assert_eq!(config.tui.theme, "light");
    assert!(!config.tui.vim_bindings);
    assert_eq!(config.registries.current, Some("local".to_string()));
    assert_eq!(config.registries.list.len(), 2);
    assert_eq!(config.registries.list[0].name, "local");
    assert!(config.registries.list[0].insecure);
    assert_eq!(config.registries.list[1].name, "prod");
    assert!(!config.registries.list[1].insecure);
}

#[test]
fn test_from_str_invalid_yaml() {
    let yaml = "output: { format: invalid }";
    let result = Config::from_yaml_str(yaml);
    assert!(result.is_err());
}

#[test]
fn test_from_str_unknown_field() {
    // config-rs should ignore unknown fields
    let yaml = "unknown_field: true";
    let result = Config::from_yaml_str(yaml);
    assert!(result.is_ok());
}
