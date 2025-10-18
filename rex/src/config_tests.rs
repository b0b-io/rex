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
