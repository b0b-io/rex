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
    let ctx = crate::context::AppContext::build(
        ColorChoice::Auto,
        crate::context::VerbosityLevel::Normal,
    );
    assert!(!should_color(&ctx));
    unsafe {
        std::env::remove_var("NO_COLOR");
    }
}

#[test]
fn test_checkmark_with_no_color() {
    unsafe {
        std::env::set_var("NO_COLOR", "1");
    }
    let ctx = crate::context::AppContext::build(
        ColorChoice::Auto,
        crate::context::VerbosityLevel::Normal,
    );
    let result = checkmark(&ctx);
    assert_eq!(result, "✓");
    unsafe {
        std::env::remove_var("NO_COLOR");
    }
}

#[test]
fn test_checkmark_returns_string() {
    // Just verify it returns a non-empty string
    let ctx = crate::context::AppContext::build(
        ColorChoice::Never,
        crate::context::VerbosityLevel::Normal,
    );
    let result = checkmark(&ctx);
    assert!(!result.is_empty());
    assert!(result.contains("✓"));
}

#[test]
fn test_error_mark_returns_string() {
    // Just verify it returns a non-empty string
    let ctx = crate::context::AppContext::build(
        ColorChoice::Never,
        crate::context::VerbosityLevel::Normal,
    );
    let result = error_mark(&ctx);
    assert!(!result.is_empty());
    assert!(result.contains("✗"));
}

#[test]
fn test_print_with_normal_verbosity_prints_normal() {
    let ctx = crate::context::AppContext::build(
        ColorChoice::Never,
        crate::context::VerbosityLevel::Normal,
    );
    // Normal messages should NOT print (use success/error/warning instead)
    // This test just verifies the function doesn't panic
    print(&ctx, crate::context::VerbosityLevel::Normal, "test");
}

#[test]
fn test_print_with_verbose_suppresses_trace() {
    let ctx = crate::context::AppContext::build(
        ColorChoice::Never,
        crate::context::VerbosityLevel::Verbose,
    );
    // At Verbose level, Trace messages should not print
    // We can't easily test stderr output, but we verify no panic
    print(
        &ctx,
        crate::context::VerbosityLevel::Trace,
        "should not print",
    );
    print(
        &ctx,
        crate::context::VerbosityLevel::Verbose,
        "should print",
    );
}

#[test]
fn test_print_with_very_verbose_suppresses_trace() {
    let ctx = crate::context::AppContext::build(
        ColorChoice::Never,
        crate::context::VerbosityLevel::VeryVerbose,
    );
    // At VeryVerbose level, Trace should not print but VeryVerbose should
    print(
        &ctx,
        crate::context::VerbosityLevel::Trace,
        "should not print",
    );
    print(
        &ctx,
        crate::context::VerbosityLevel::VeryVerbose,
        "should print",
    );
    print(
        &ctx,
        crate::context::VerbosityLevel::Verbose,
        "should also print",
    );
}

#[test]
fn test_print_with_trace_prints_everything() {
    let ctx = crate::context::AppContext::build(
        ColorChoice::Never,
        crate::context::VerbosityLevel::Trace,
    );
    // At Trace level, all messages should print
    print(&ctx, crate::context::VerbosityLevel::Trace, "should print");
    print(
        &ctx,
        crate::context::VerbosityLevel::VeryVerbose,
        "should print",
    );
    print(
        &ctx,
        crate::context::VerbosityLevel::Verbose,
        "should print",
    );
    print(&ctx, crate::context::VerbosityLevel::Normal, "should print");
}

#[test]
fn test_print_respects_verbosity_hierarchy() {
    // Normal suppresses all verbose output
    let ctx_normal = crate::context::AppContext::build(
        ColorChoice::Never,
        crate::context::VerbosityLevel::Normal,
    );
    // These should not print (but we can't test stderr easily, just verify no panic)
    print(
        &ctx_normal,
        crate::context::VerbosityLevel::Verbose,
        "suppressed",
    );
    print(
        &ctx_normal,
        crate::context::VerbosityLevel::VeryVerbose,
        "suppressed",
    );
    print(
        &ctx_normal,
        crate::context::VerbosityLevel::Trace,
        "suppressed",
    );
}
