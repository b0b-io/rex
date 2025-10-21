use super::*;
use std::env;

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.style.format, OutputFormat::Pretty);
    assert!(config.style.color);
}

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("[style]"));
    assert!(toml_str.contains("format"));
    assert!(toml_str.contains("color"));
}

#[test]
fn test_config_deserialization() {
    let toml_str = r#"
[style]
format = "json"
color = false
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.style.format, OutputFormat::Json);
    assert!(!config.style.color);
}

#[test]
fn test_config_load_from_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.style.format, config.style.format);
    assert_eq!(loaded.style.color, config.style.color);
}

#[test]
fn test_config_load_nonexistent_file() {
    let config_path = std::path::PathBuf::from("/tmp/nonexistent_config.toml");
    let result = Config::load(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_config_save_creates_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("subdir").join("config.toml");

    let config = Config::default();
    let result = config.save(&config_path);
    assert!(result.is_ok());
    assert!(config_path.exists());
}

#[test]
fn test_get_config_path_uses_env_var() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("custom_config.toml");

    unsafe {
        env::set_var("REX_CONFIG", config_path.to_str().unwrap());
    }
    let result = get_config_path();
    unsafe {
        env::remove_var("REX_CONFIG");
    }

    assert_eq!(result, config_path);
}

#[test]
fn test_get_config_path_default() {
    unsafe {
        env::remove_var("REX_CONFIG");
    }
    let result = get_config_path();

    // Should return the default path in user's config directory
    assert!(
        result.to_str().unwrap().contains("config") || result.to_str().unwrap().contains(".config")
    );
}

#[test]
fn test_style_config_defaults() {
    let style = StyleConfig::default();
    assert_eq!(style.format, OutputFormat::Pretty);
    assert!(style.color);
}

#[test]
fn test_init_config_creates_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let result = init_config(&config_path);
    assert!(result.is_ok());
    assert!(config_path.exists());

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.style.format, OutputFormat::Pretty);
}

#[test]
fn test_init_config_fails_if_exists() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Create config first
    init_config(&config_path).unwrap();

    // Try to init again
    let result = init_config(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_get_config_value_nested() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = get_config_value(&config_path, "style.format");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "pretty");
}

#[test]
fn test_get_config_value_nonexistent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = get_config_value(&config_path, "nonexistent.key");
    assert!(result.is_err());
}

#[test]
fn test_set_config_value_nested() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = set_config_value(&config_path, "style.format", "json");
    assert!(result.is_ok());

    let value = get_config_value(&config_path, "style.format").unwrap();
    assert_eq!(value, "json");
}

#[test]
fn test_set_config_value_creates_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let result = set_config_value(&config_path, "style.format", "yaml");
    assert!(result.is_ok());
    assert!(config_path.exists());
}

#[test]
fn test_edit_config_no_editor() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    unsafe {
        env::remove_var("EDITOR");
        env::remove_var("VISUAL");
    }

    let result = edit_config(&config_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No editor found"));
}

#[test]
fn test_display_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = display_config(&config_path);
    assert!(result.is_ok());
}

#[test]
fn test_registries_config_default() {
    let registries = RegistriesConfig::default();
    assert!(registries.default.is_none());
    assert!(registries.list.is_empty());
}

#[test]
fn test_registry_entry_creation() {
    let entry = RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    };
    assert_eq!(entry.name, "local");
    assert_eq!(entry.url, "http://localhost:5000");
}

#[test]
fn test_config_with_registries_serialization() {
    let mut config = Config::default();
    config.registries.default = Some("dockerhub".to_string());
    config.registries.list.push(RegistryEntry {
        name: "dockerhub".to_string(),
        url: "https://registry-1.docker.io".to_string(),
    });
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });

    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("[registries]"));
    assert!(toml_str.contains("default = \"dockerhub\""));
    assert!(toml_str.contains("[[registries.list]]"));
    assert!(toml_str.contains("name = \"dockerhub\""));
    assert!(toml_str.contains("name = \"local\""));
}

#[test]
fn test_config_with_registries_deserialization() {
    let toml_str = r#"
[style]
format = "pretty"
color = true

[registries]
default = "dockerhub"

[[registries.list]]
name = "dockerhub"
url = "https://registry-1.docker.io"

[[registries.list]]
name = "local"
url = "http://localhost:5000"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.registries.default, Some("dockerhub".to_string()));
    assert_eq!(config.registries.list.len(), 2);
    assert_eq!(config.registries.list[0].name, "dockerhub");
    assert_eq!(config.registries.list[1].name, "local");
}

#[test]
fn test_config_with_empty_registries() {
    let config = Config::default();

    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    config.save(&config_path).unwrap();

    let loaded = Config::load(&config_path).unwrap();
    assert!(loaded.registries.default.is_none());
    assert!(loaded.registries.list.is_empty());
}

#[test]
fn test_registry_entry_equality() {
    let entry1 = RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    };
    let entry2 = RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    };
    let entry3 = RegistryEntry {
        name: "remote".to_string(),
        url: "http://example.com".to_string(),
    };

    assert_eq!(entry1, entry2);
    assert_ne!(entry1, entry3);
}

// Tests for registry add command
#[test]
fn test_add_registry_to_empty_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = add_registry(&config_path, "local", "http://localhost:5000");
    assert!(result.is_ok());

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.registries.list.len(), 1);
    assert_eq!(loaded.registries.list[0].name, "local");
    assert_eq!(loaded.registries.list[0].url, "http://localhost:5000/");
}

#[test]
fn test_add_registry_to_existing_registries() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.list.push(RegistryEntry {
        name: "existing".to_string(),
        url: "http://example.com".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = add_registry(&config_path, "local", "http://localhost:5000");
    assert!(result.is_ok());

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.registries.list.len(), 2);
    assert_eq!(loaded.registries.list[1].name, "local");
}

#[test]
fn test_add_registry_duplicate_name_fails() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = add_registry(&config_path, "local", "http://other:5000");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_add_registry_creates_config_if_not_exists() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let result = add_registry(&config_path, "local", "http://localhost:5000");
    assert!(result.is_ok());
    assert!(config_path.exists());

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.registries.list.len(), 1);
}

#[test]
fn test_add_registry_normalizes_url() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = add_registry(&config_path, "local", "localhost:5000");
    assert!(result.is_ok());

    let loaded = Config::load(&config_path).unwrap();
    // Should normalize to http://localhost:5000
    assert!(loaded.registries.list[0].url.starts_with("http://"));
}

#[test]
fn test_add_first_registry_sets_default() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = add_registry(&config_path, "local", "http://localhost:5000");
    assert!(result.is_ok());

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.registries.default, Some("local".to_string()));
}

#[test]
fn test_add_second_registry_does_not_change_default() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.default = Some("first".to_string());
    config.registries.list.push(RegistryEntry {
        name: "first".to_string(),
        url: "http://example.com".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = add_registry(&config_path, "second", "http://localhost:5000");
    assert!(result.is_ok());

    let loaded = Config::load(&config_path).unwrap();
    // Should still be "first"
    assert_eq!(loaded.registries.default, Some("first".to_string()));
}

// Tests for registry list command
#[test]
fn test_list_registries_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = list_registries(&config_path);
    assert!(result.is_ok());
    let registries = result.unwrap();
    assert!(registries.is_empty());
}

#[test]
fn test_list_registries_with_entries() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.registries.list.push(RegistryEntry {
        name: "dockerhub".to_string(),
        url: "https://registry-1.docker.io".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = list_registries(&config_path);
    assert!(result.is_ok());
    let registries = result.unwrap();
    assert_eq!(registries.len(), 2);
    assert_eq!(registries[0].name, "local");
    assert_eq!(registries[1].name, "dockerhub");
}

#[test]
fn test_list_registries_nonexistent_config() {
    let config_path = std::path::PathBuf::from("/tmp/nonexistent_registry_config.toml");
    let result = list_registries(&config_path);
    assert!(result.is_err());
}

// Tests for URL validation
#[test]
fn test_validate_registry_url_with_http_scheme() {
    let result = validate_registry_url("http://localhost:5000");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "http://localhost:5000/");
}

#[test]
fn test_validate_registry_url_with_https_scheme() {
    let result = validate_registry_url("https://registry-1.docker.io");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://registry-1.docker.io/");
}

#[test]
fn test_validate_registry_url_without_scheme() {
    let result = validate_registry_url("localhost:5000");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "http://localhost:5000/");
}

#[test]
fn test_validate_registry_url_domain_without_port() {
    let result = validate_registry_url("registry.example.com");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "http://registry.example.com/");
}

#[test]
fn test_validate_registry_url_with_path() {
    let result = validate_registry_url("https://example.com/v2/");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://example.com/v2/");
}

#[test]
fn test_validate_registry_url_malformed_double_colon() {
    let result = validate_registry_url("http:://badurl");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid URL"));
}

#[test]
fn test_validate_registry_url_invalid_scheme_ftp() {
    let result = validate_registry_url("ftp://example.com");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid URL scheme 'ftp'"));
}

#[test]
fn test_validate_registry_url_invalid_scheme_file() {
    let result = validate_registry_url("file:///tmp/registry");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid URL scheme 'file'"));
}

#[test]
fn test_validate_registry_url_empty_string() {
    let result = validate_registry_url("");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid URL"));
}

#[test]
fn test_validate_registry_url_only_scheme() {
    let result = validate_registry_url("http://");
    assert!(result.is_err());
    // The url crate parses "http://" successfully with an empty host
    // So we just check that it's an error
    let err = result.unwrap_err();
    assert!(err.contains("Invalid URL") || err.contains("missing host"));
}

#[test]
fn test_validate_registry_url_spaces() {
    let result = validate_registry_url("http://bad url.com");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid URL"));
}

#[test]
fn test_validate_registry_url_ipv4_address() {
    let result = validate_registry_url("192.168.1.100:5000");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "http://192.168.1.100:5000/");
}

#[test]
fn test_validate_registry_url_ipv6_address() {
    let result = validate_registry_url("http://[::1]:5000");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "http://[::1]:5000/");
}

// Tests for registry remove command
#[test]
fn test_remove_registry_existing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.registries.list.push(RegistryEntry {
        name: "dockerhub".to_string(),
        url: "https://registry-1.docker.io".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = remove_registry(&config_path, "local");
    assert!(result.is_ok());

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.registries.list.len(), 1);
    assert_eq!(loaded.registries.list[0].name, "dockerhub");
}

#[test]
fn test_remove_registry_nonexistent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = remove_registry(&config_path, "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_remove_last_registry() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.default = Some("local".to_string());
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = remove_registry(&config_path, "local");
    assert!(result.is_ok());

    let loaded = Config::load(&config_path).unwrap();
    assert!(loaded.registries.list.is_empty());
    assert!(loaded.registries.default.is_none());
}

#[test]
fn test_remove_default_registry_clears_default() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.default = Some("local".to_string());
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.registries.list.push(RegistryEntry {
        name: "dockerhub".to_string(),
        url: "https://registry-1.docker.io".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = remove_registry(&config_path, "local");
    assert!(result.is_ok());

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.registries.list.len(), 1);
    assert!(loaded.registries.default.is_none());
}

#[test]
fn test_remove_registry_empty_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = remove_registry(&config_path, "local");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// Tests for registry use command
#[test]
fn test_use_registry_existing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.default = Some("dockerhub".to_string());
    config.registries.list.push(RegistryEntry {
        name: "dockerhub".to_string(),
        url: "https://registry-1.docker.io".to_string(),
    });
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = use_registry(&config_path, "local");
    assert!(result.is_ok());

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.registries.default, Some("local".to_string()));
}

#[test]
fn test_use_registry_nonexistent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = use_registry(&config_path, "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_use_registry_when_none_set() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = use_registry(&config_path, "local");
    assert!(result.is_ok());

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.registries.default, Some("local".to_string()));
}

#[test]
fn test_use_registry_empty_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = use_registry(&config_path, "local");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// Tests for registry show command
#[test]
fn test_show_registry_existing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = show_registry(&config_path, "local");
    assert!(result.is_ok());
    let registry = result.unwrap();
    assert_eq!(registry.name, "local");
    assert_eq!(registry.url, "http://localhost:5000");
}

#[test]
fn test_show_registry_nonexistent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = show_registry(&config_path, "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_show_registry_with_default_marker() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.registries.default = Some("local".to_string());
    config.registries.list.push(RegistryEntry {
        name: "local".to_string(),
        url: "http://localhost:5000".to_string(),
    });
    config.save(&config_path).unwrap();

    let result = show_registry(&config_path, "local");
    assert!(result.is_ok());
    let display = result.unwrap();
    assert_eq!(display.name, "local");
    assert_eq!(display.default, "*");
}

#[test]
fn test_show_registry_empty_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = show_registry(&config_path, "local");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// Tests for registry check command
#[tokio::test]
async fn test_check_registry_nonexistent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = Config::default();
    config.save(&config_path).unwrap();

    let result = super::check_registry(&config_path, "nonexistent").await;
    assert!(!result.online);
    assert!(result.error.unwrap().contains("not found in configuration"));
}

#[tokio::test]
async fn test_check_registry_invalid_config() {
    let config_path = std::path::PathBuf::from("/tmp/nonexistent_check_config.toml");

    let result = super::check_registry(&config_path, "test").await;
    assert!(!result.online);
    assert!(result.error.unwrap().contains("Configuration error"));
}

#[test]
fn test_registry_check_result_creation() {
    let result = RegistryCheckResult {
        name: "test".to_string(),
        url: "http://localhost:5000".to_string(),
        online: true,
        auth_required: false,
        authenticated: false,
        api_version: Some("registry/2.0".to_string()),
        error: None,
    };

    assert_eq!(result.name, "test");
    assert!(result.online);
    assert!(!result.auth_required);
    assert!(!result.authenticated);
    assert_eq!(result.api_version, Some("registry/2.0".to_string()));
    assert!(result.error.is_none());
}

#[test]
fn test_registry_check_result_with_error() {
    let result = RegistryCheckResult {
        name: "test".to_string(),
        url: "http://localhost:5000".to_string(),
        online: false,
        auth_required: false,
        authenticated: false,
        api_version: None,
        error: Some("Connection refused".to_string()),
    };

    assert!(!result.online);
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "Connection refused");
}

#[test]
fn test_registry_check_result_pretty_format_online() {
    let result = RegistryCheckResult {
        name: "test".to_string(),
        url: "http://localhost:5000".to_string(),
        online: true,
        auth_required: false,
        authenticated: false,
        api_version: Some("registry/2.0".to_string()),
        error: None,
    };

    let output = result.format_pretty();
    assert!(output.contains("Registry: test"));
    assert!(output.contains("URL: http://localhost:5000"));
    assert!(output.contains("Status: ✓ Online"));
    assert!(output.contains("API Version: registry/2.0"));
    assert!(output.contains("Authentication: ○ Not required"));
}

#[test]
fn test_registry_check_result_pretty_format_offline() {
    let result = RegistryCheckResult {
        name: "test".to_string(),
        url: "http://localhost:5000".to_string(),
        online: false,
        auth_required: false,
        authenticated: false,
        api_version: None,
        error: Some("Connection refused".to_string()),
    };

    let output = result.format_pretty();
    assert!(output.contains("Registry: test"));
    assert!(output.contains("Status: ✗ Offline"));
    assert!(output.contains("Reason: Connection refused"));
}

#[test]
fn test_registry_check_result_pretty_format_auth_required() {
    let result = RegistryCheckResult {
        name: "dockerhub".to_string(),
        url: "https://registry-1.docker.io".to_string(),
        online: false,
        auth_required: true,
        authenticated: false,
        api_version: None,
        error: Some("Authentication required".to_string()),
    };

    let output = result.format_pretty();
    assert!(output.contains("Status: ✗ Offline"));
    assert!(output.contains("Reason: Authentication required"));
}

#[test]
fn test_registry_check_result_pretty_format_authenticated() {
    let result = RegistryCheckResult {
        name: "test".to_string(),
        url: "http://localhost:5000".to_string(),
        online: true,
        auth_required: false,
        authenticated: true,
        api_version: Some("registry/2.0".to_string()),
        error: None,
    };

    let output = result.format_pretty();
    assert!(output.contains("Authentication: ✓ Authenticated"));
}

#[test]
fn test_registry_check_result_pretty_format_auth_required_not_authenticated() {
    let result = RegistryCheckResult {
        name: "test".to_string(),
        url: "http://localhost:5000".to_string(),
        online: true,
        auth_required: true,
        authenticated: false,
        api_version: Some("registry/2.0".to_string()),
        error: None,
    };

    let output = result.format_pretty();
    assert!(output.contains("Authentication: ⚠ Required (not configured)"));
}

#[test]
fn test_registry_check_result_serialization() {
    let result = RegistryCheckResult {
        name: "test".to_string(),
        url: "http://localhost:5000".to_string(),
        online: true,
        auth_required: false,
        authenticated: false,
        api_version: Some("registry/2.0".to_string()),
        error: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"name\":\"test\""));
    assert!(json.contains("\"online\":true"));
    assert!(json.contains("\"auth_required\":false"));
    assert!(json.contains("\"authenticated\":false"));
    assert!(json.contains("\"api_version\":\"registry/2.0\""));
}
