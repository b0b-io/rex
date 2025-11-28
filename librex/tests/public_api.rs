use librex::RexBuilder;

#[test]
fn test_rex_builder_new() {
    let _builder = RexBuilder::new().registry_url("http://localhost:5000");
    // We can't assert fields directly as they will be private,
    // so this test just ensures the builder can be created.
    // More detailed tests will be on the `build` method.
}

#[test]
fn test_rex_builder_with_dockerhub_compat() {
    // Test that with_dockerhub_compat() can be chained
    let _builder = RexBuilder::new()
        .registry_url("http://localhost:5000")
        .with_dockerhub_compat(true);

    // Test with false
    let _builder2 = RexBuilder::new()
        .registry_url("http://localhost:5000")
        .with_dockerhub_compat(false);

    // This test ensures the method exists and can be called
}
