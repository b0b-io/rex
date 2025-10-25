use super::*;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
struct TestData {
    name: String,
    value: i32,
}

impl Formattable for TestData {
    fn format_pretty(&self) -> String {
        format!("{}: {}", self.name, self.value)
    }
}

#[test]
fn test_output_format_from_string() {
    assert_eq!(OutputFormat::from("pretty"), OutputFormat::Pretty);
    assert_eq!(OutputFormat::from("json"), OutputFormat::Json);
    assert_eq!(OutputFormat::from("yaml"), OutputFormat::Yaml);
    assert_eq!(OutputFormat::from("invalid"), OutputFormat::Pretty);
}

#[test]
fn test_format_pretty() {
    let data = TestData {
        name: "test".to_string(),
        value: 42,
    };
    let result = format_output(&data, OutputFormat::Pretty);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test: 42");
}

#[test]
fn test_format_json() {
    let data = TestData {
        name: "test".to_string(),
        value: 42,
    };
    let result = format_output(&data, OutputFormat::Json);
    assert!(result.is_ok());
    let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(json["name"], "test");
    assert_eq!(json["value"], 42);
}

#[test]
fn test_format_yaml() {
    let data = TestData {
        name: "test".to_string(),
        value: 42,
    };
    let result = format_output(&data, OutputFormat::Yaml);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("name: test"));
    assert!(output.contains("value: 42"));
}

#[test]
fn test_format_vec_pretty() {
    let data = vec![
        TestData {
            name: "first".to_string(),
            value: 1,
        },
        TestData {
            name: "second".to_string(),
            value: 2,
        },
    ];
    let result = format_output_vec(&data, OutputFormat::Pretty);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("first: 1"));
    assert!(output.contains("second: 2"));
}

#[test]
fn test_format_vec_json() {
    let data = vec![
        TestData {
            name: "first".to_string(),
            value: 1,
        },
        TestData {
            name: "second".to_string(),
            value: 2,
        },
    ];
    let result = format_output_vec(&data, OutputFormat::Json);
    assert!(result.is_ok());
    let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 2);
}

#[test]
fn test_format_empty_vec() {
    let data: Vec<TestData> = vec![];
    let result = format_output_vec(&data, OutputFormat::Pretty);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[test]
fn test_should_color_respects_no_color_env() {
    // Set NO_COLOR environment variable
    unsafe {
        std::env::set_var("NO_COLOR", "1");
    }
    assert!(!should_color());
    unsafe {
        std::env::remove_var("NO_COLOR");
    }
}

#[test]
fn test_checkmark_with_no_color() {
    unsafe {
        std::env::set_var("NO_COLOR", "1");
    }
    let result = checkmark();
    assert_eq!(result, "✓");
    unsafe {
        std::env::remove_var("NO_COLOR");
    }
}

#[test]
fn test_checkmark_returns_string() {
    // Just verify it returns a non-empty string
    let result = checkmark();
    assert!(!result.is_empty());
    assert!(result.contains("✓"));
}

#[test]
fn test_error_mark_returns_string() {
    // Just verify it returns a non-empty string
    let result = error_mark();
    assert!(!result.is_empty());
    assert!(result.contains("✗"));
}
