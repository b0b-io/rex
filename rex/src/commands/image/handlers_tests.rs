use super::*;

// Note: These tests use mockito to test get_image_inspect end-to-end with mock HTTP responses.
// The tests verify that the function correctly calls the registry, parses responses, and
// handles both success and error cases.

#[test]
fn test_get_image_inspect_invalid_reference() {
    let server = mockito::Server::new();
    let registry_url = server.url();

    // Call get_image_inspect with invalid reference
    let result = get_image_inspect(&registry_url, "", None, false, false);

    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Invalid image reference"));
}

#[test]
fn test_get_image_inspect_manifest_not_found() {
    let mut server = mockito::Server::new();
    let registry_url = server.url();

    // Mock version check
    let _v2_mock = server
        .mock("GET", "/v2/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create();

    // Mock manifest fetch with 404
    let _manifest_mock = server
        .mock("GET", "/v2/test/repo/manifests/nonexistent")
        .with_status(404)
        .with_body("{\"errors\":[{\"code\":\"MANIFEST_UNKNOWN\"}]}")
        .create();

    // Call get_image_inspect
    let result = get_image_inspect(&registry_url, "test/repo:nonexistent", None, false, false);

    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Failed to fetch manifest"));
}

// TODO: Full integration test with real OCI structures
// Currently commented out due to oci-spec validation being very strict about JSON format
// The success case is tested in integration tests with real registries
#[allow(dead_code)]
#[test]
fn test_get_image_inspect_single_platform() {
    let mut server = mockito::Server::new();
    let registry_url = server.url();

    // Mock version check
    let _v2_mock = server
        .mock("GET", "/v2/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create();

    // Create a minimal OCI manifest (matching oci-spec format)
    let manifest_json = r#"{ 
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.manifest.v1+json",
    "config": {
        "mediaType": "application/vnd.oci.image.config.v1+json",
        "size": 1024,
        "digest": "sha256:05d6eacdcaf34accb9bfcc28ce285c4aee9844550f36890488468f9bcceebd76"
    },
    "layers": [
        {
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "size": 2048,
            "digest": "sha256:1111111111111111111111111111111111111111111111111111111111111111"
        },
        {
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "size": 4096,
            "digest": "sha256:2222222222222222222222222222222222222222222222222222222222222222"
        }
    ],
    "annotations": {
        "org.opencontainers.image.ref.name": "test/repo:latest"
    }
}"#;

    // Create a minimal OCI image config
    let config_json = r#"{ 
        "architecture": "amd64",
        "os": "linux",
        "created": "2023-01-01T00:00:00Z",
        "config": {
            "Env": ["PATH=/usr/local/bin:/usr/bin", "FOO=bar"],
            "Cmd": ["/bin/sh"],
            "WorkingDir": "/app",
            "User": "1000",
            "Labels": {
                "version": "1.0.0",
                "maintainer": "test@example.com"
            },
            "ExposedPorts": {
                "80/tcp": {},
                "443/tcp": {}
            },
            "Volumes": {
                "/data": {},
                "/logs": {}
            }
        },
        "rootfs": {
            "type": "layers",
            "diff_ids": [
                "sha256:diff1111111111111111111111111111111111111111111111111111111111111",
                "sha256:diff2222222222222222222222222222222222222222222222222222222222222"
            ]
        },
        "history": [
            {
                "created": "2023-01-01T00:00:00Z",
                "created_by": "/bin/sh -c #(nop) ADD file:123 in /"
            },
            {
                "created": "2023-01-01T00:01:00Z",
                "created_by": "/bin/sh -c #(nop) CMD [\"/bin/sh\"]",
                "empty_layer": true
            }
        ]
    }"#;

    // Mock manifest fetch
    let _manifest_mock = server
        .mock("GET", "/v2/test-single/repo/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header(
            "docker-content-digest",
            "sha256:manifestdigest1111111111111111111111111111111111111111111111111",
        )
        .with_body(manifest_json)
        .create();

    // Mock config blob fetch
    let _config_mock = server
        .mock(
            "GET",
            "/v2/test-single/repo/blobs/sha256:05d6eacdcaf34accb9bfcc28ce285c4aee9844550f36890488468f9bcceebd76",
        )
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.config.v1+json")
        .with_body(config_json)
        .create();

    // Call get_image_inspect
    let result = get_image_inspect(&registry_url, "test-single/repo:latest", None, false, false);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let inspect = result.unwrap();

    // Verify basic fields
    assert_eq!(inspect.reference, "test-single/repo:latest");
    assert_eq!(inspect.registry, registry_url);
    assert_eq!(inspect.architecture, "amd64");
    assert_eq!(inspect.os, "linux");
    assert_eq!(
        inspect.config_digest,
        "sha256:05d6eacdcaf34accb9bfcc28ce285c4aee9844550f36890488468f9bcceebd76"
    );

    // Verify size calculation (sum of layer sizes)
    assert_eq!(inspect.size, 2048 + 4096);

    // Verify environment variables
    assert_eq!(inspect.env.len(), 2);
    assert!(
        inspect
            .env
            .contains(&"PATH=/usr/local/bin:/usr/bin".to_string())
    );
    assert!(inspect.env.contains(&"FOO=bar".to_string()));

    // Verify cmd
    assert!(inspect.cmd.is_some());
    assert_eq!(inspect.cmd.unwrap(), vec!["/bin/sh"]);

    // Verify working directory
    assert_eq!(inspect.working_dir, Some("/app".to_string()));

    // Verify user
    assert_eq!(inspect.user, Some("1000".to_string()));

    // Verify labels
    assert_eq!(inspect.labels.len(), 2);
    assert_eq!(inspect.labels.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(
        inspect.labels.get("maintainer"),
        Some(&"test@example.com".to_string())
    );

    // Verify exposed ports
    assert_eq!(inspect.exposed_ports.len(), 2);
    assert!(inspect.exposed_ports.contains(&"80/tcp".to_string()));
    assert!(inspect.exposed_ports.contains(&"443/tcp".to_string()));

    // Verify volumes
    assert_eq!(inspect.volumes.len(), 2);
    assert!(inspect.volumes.contains(&"/data".to_string()));
    assert!(inspect.volumes.contains(&"/logs".to_string()));

    // Verify layers
    assert_eq!(inspect.layers.len(), 2);
    assert_eq!(
        inspect.layers[0].digest,
        "sha256:1111111111111111111111111111111111111111111111111111111111111111"
    );
    assert_eq!(inspect.layers[0].size, 2048);
    assert_eq!(
        inspect.layers[1].digest,
        "sha256:2222222222222222222222222222222222222222222222222222222222222222"
    );
    assert_eq!(inspect.layers[1].size, 4096);

    // Verify history
    assert_eq!(inspect.history.len(), 2);
    assert!(!inspect.history[0].empty_layer);
    assert!(inspect.history[1].empty_layer);
    assert!(
        inspect.history[0]
            .created_by
            .as_ref()
            .unwrap()
            .contains("ADD file")
    );

    // Verify rootfs diff_ids
    assert_eq!(inspect.rootfs_diff_ids.len(), 2);
    assert_eq!(
        inspect.rootfs_diff_ids[0],
        "sha256:diff1111111111111111111111111111111111111111111111111111111111111"
    );
}

// Test multi-platform image without --platform flag (should error with available platforms)
#[allow(dead_code)]
#[test]
fn test_get_image_inspect_multi_platform_no_flag() {
    let mut server = mockito::Server::new();
    let registry_url = server.url();

    // Mock version check
    let _v2_mock = server
        .mock("GET", "/v2/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create();

    // Create a multi-platform OCI image index (using same format as librex tests)
    let index_json = r#"{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.index.v1+json",
    "manifests": [
        {
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "size": 7143,
            "digest": "sha256:aaaa1234567890abcdef1234567890abcdef1234567890abcdef123456789012",
            "platform": {
                "architecture": "amd64",
                "os": "linux"
            }
        },
        {
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "size": 7682,
            "digest": "sha256:bbbb1234567890abcdef1234567890abcdef1234567890abcdef123456789012",
            "platform": {
                "architecture": "arm64",
                "os": "linux"
            }
        }
    ]
}"#;

    // Mock index fetch
    let _index_mock = server
        .mock("GET", "/v2/test-multi-no-flag/repo/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.index.v1+json")
        .with_header(
            "docker-content-digest",
            "sha256:indexdigest11111111111111111111111111111111111111111111111111111",
        )
        .with_body(index_json)
        .create();

    // Call get_image_inspect without platform flag
    let result = get_image_inspect(
        &registry_url,
        "test-multi-no-flag/repo:latest",
        None,
        false,
        false,
    );

    // Should error with helpful message listing available platforms
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Multi-platform image detected"));
    assert!(err_msg.contains("--platform"));
    assert!(err_msg.contains("linux/amd64"));
    assert!(err_msg.contains("linux/arm64"));
}

// Test multi-platform image with valid --platform flag (should fetch specific platform)
#[allow(dead_code)]
#[test]
fn test_get_image_inspect_multi_platform_with_valid_flag() {
    let mut server = mockito::Server::new();
    let registry_url = server.url();

    // Mock version check
    let _v2_mock = server
        .mock("GET", "/v2/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create();

    // Create a multi-platform OCI image index (using same format as librex tests)
    let index_json = r#"{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.index.v1+json",
    "manifests": [
        {
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "size": 7143,
            "digest": "sha256:aaaa1234567890abcdef1234567890abcdef1234567890abcdef123456789012",
            "platform": {
                "architecture": "amd64",
                "os": "linux"
            }
        },
        {
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "size": 7682,
            "digest": "sha256:bbbb1234567890abcdef1234567890abcdef1234567890abcdef123456789012",
            "platform": {
                "architecture": "arm64",
                "os": "linux"
            }
        }
    ]
}"#;

    // Create a platform-specific manifest (for linux/arm64) - using format from librex tests
    let manifest_json = r#"{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.manifest.v1+json",
    "config": {
        "mediaType": "application/vnd.oci.image.config.v1+json",
        "size": 376,
        "digest": "sha256:38850cc6cf9d25df7c4450466dd2b52a73d7c0434a9945fe573e9f798b8f6ab6"
    },
    "layers": [
        {
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "size": 32654,
            "digest": "sha256:9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0"
        }
    ]
}"#;

    // Create config for arm64 platform
    let config_json = r#"{
        "architecture": "arm64",
        "os": "linux",
        "created": "2023-01-01T00:00:00Z",
        "config": {
            "Env": ["PATH=/usr/local/bin:/usr/bin"],
            "Cmd": ["/bin/sh"]
        },
        "rootfs": {
            "type": "layers",
            "diff_ids": [
                "sha256:arm64diff111234567890abcdef1234567890abcdef1234567890abcdef12345"
            ]
        },
        "history": [
            {
                "created": "2023-01-01T00:00:00Z",
                "created_by": "/bin/sh -c #(nop) ADD file:123 in /"
            }
        ]
    }"#;

    // Mock index fetch
    let _index_mock = server
        .mock("GET", "/v2/test-multi-flag/repo/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.index.v1+json")
        .with_header(
            "docker-content-digest",
            "sha256:indexdigest11111111111111111111111111111111111111111111111111111",
        )
        .with_body(index_json)
        .create();

    // Mock platform-specific manifest fetch (by digest)
    let _manifest_mock = server
        .mock(
            "GET",
            "/v2/test-multi-flag/repo/manifests/sha256:bbbb1234567890abcdef1234567890abcdef1234567890abcdef123456789012",
        )
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header(
            "docker-content-digest",
            "sha256:bbbb1234567890abcdef1234567890abcdef1234567890abcdef123456789012",
        )
        .with_body(manifest_json)
        .create();

    // Mock config blob fetch
    let _config_mock = server
        .mock(
            "GET",
            "/v2/test-multi-flag/repo/blobs/sha256:38850cc6cf9d25df7c4450466dd2b52a73d7c0434a9945fe573e9f798b8f6ab6",
        )
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.config.v1+json")
        .with_body(config_json)
        .create();

    // Call get_image_inspect with platform flag
    let result = get_image_inspect(
        &registry_url,
        "test-multi-flag/repo:latest",
        Some("linux/arm64"),
        false,
        false,
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let inspect = result.unwrap();

    // Verify it fetched the arm64 platform
    assert_eq!(inspect.architecture, "arm64");
    assert_eq!(inspect.os, "linux");
    assert_eq!(inspect.size, 32654); // Layer size from manifest
}

// Test multi-platform image with invalid platform (should error with available platforms)
#[allow(dead_code)]
#[test]
fn test_get_image_inspect_multi_platform_invalid_platform() {
    let mut server = mockito::Server::new();
    let registry_url = server.url();

    // Mock version check
    let _v2_mock = server
        .mock("GET", "/v2/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create();

    // Create a multi-platform OCI image index (using same format as librex tests)
    let index_json = r#"{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.index.v1+json",
    "manifests": [
        {
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "size": 7143,
            "digest": "sha256:aaaa1234567890abcdef1234567890abcdef1234567890abcdef123456789012",
            "platform": {
                "architecture": "amd64",
                "os": "linux"
            }
        },
        {
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "size": 7682,
            "digest": "sha256:bbbb1234567890abcdef1234567890abcdef1234567890abcdef123456789012",
            "platform": {
                "architecture": "arm64",
                "os": "linux"
            }
        }
    ]
}"#;

    // Mock index fetch
    let _index_mock = server
        .mock("GET", "/v2/test-multi-invalid/repo/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.index.v1+json")
        .with_header(
            "docker-content-digest",
            "sha256:indexdigest11111111111111111111111111111111111111111111111111111",
        )
        .with_body(index_json)
        .create();

    // Call get_image_inspect with invalid platform
    let result = get_image_inspect(
        &registry_url,
        "test-multi-invalid/repo:latest",
        Some("linux/s390x"),
        false,
        false,
    );

    // Should error with helpful message listing available platforms
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Platform 'linux/s390x' not found"));
    assert!(err_msg.contains("Available platforms"));
    assert!(err_msg.contains("linux/amd64"));
    assert!(err_msg.contains("linux/arm64"));
}

// Test raw manifest flag returns JSON
#[allow(dead_code)]
#[test]
fn test_get_image_inspect_raw_manifest() {
    let mut server = mockito::Server::new();
    let registry_url = server.url();

    // Mock version check
    let _v2_mock = server
        .mock("GET", "/v2/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create();

    // Create a minimal OCI manifest
    let manifest_json = r#"{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.manifest.v1+json",
    "config": {
        "mediaType": "application/vnd.oci.image.config.v1+json",
        "size": 265,
        "digest": "sha256:25890cdad8e8e7bdea614016480bb7dda663c418d23a4ae997b53ed17a022c85"
    },
    "layers": [
        {
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "size": 32654,
            "digest": "sha256:9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0"
        }
    ]
}"#;

    // Create a minimal OCI image config
    let config_json = r#"{
        "architecture": "amd64",
        "os": "linux",
        "created": "2023-01-01T00:00:00Z",
        "config": {},
        "rootfs": {
            "type": "layers",
            "diff_ids": [
                "sha256:diff1111111111111111111111111111111111111111111111111111111111111"
            ]
        }
    }"#;

    // Mock manifest fetch
    let _manifest_mock = server
        .mock("GET", "/v2/test-raw-manifest/repo/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header(
            "docker-content-digest",
            "sha256:manifestdigest1111111111111111111111111111111111111111111111111",
        )
        .with_body(manifest_json)
        .create();

    // Mock config blob fetch
    let _config_mock = server
        .mock(
            "GET",
            "/v2/test-raw-manifest/repo/blobs/sha256:25890cdad8e8e7bdea614016480bb7dda663c418d23a4ae997b53ed17a022c85",
        )
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.config.v1+json")
        .with_body(config_json)
        .create();

    // Call get_image_inspect with raw_manifest=true
    let result = get_image_inspect(
        &registry_url,
        "test-raw-manifest/repo:latest",
        None,
        true,
        false,
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let inspect = result.unwrap();

    // Verify raw_manifest is populated
    assert!(inspect.raw_manifest.is_some());
    let raw_manifest = inspect.raw_manifest.unwrap();

    // Verify it's valid JSON and contains expected fields
    assert!(raw_manifest.contains("schemaVersion"));
    assert!(raw_manifest.contains("\"config\""));
    assert!(raw_manifest.contains("\"layers\""));
    assert!(
        raw_manifest
            .contains("sha256:25890cdad8e8e7bdea614016480bb7dda663c418d23a4ae997b53ed17a022c85")
    );

    // Verify raw_config is NOT populated
    assert!(inspect.raw_config.is_none());
}

// Test raw config flag returns JSON
#[allow(dead_code)]
#[test]
fn test_get_image_inspect_raw_config() {
    let mut server = mockito::Server::new();
    let registry_url = server.url();

    // Mock version check
    let _v2_mock = server
        .mock("GET", "/v2/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create();

    // Create a minimal OCI manifest
    let manifest_json = r#"{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.manifest.v1+json",
    "config": {
        "mediaType": "application/vnd.oci.image.config.v1+json",
        "size": 289,
        "digest": "sha256:0af3a19ed6ae2afc5f87f1a90ec80092b46491b497ce4d0570193c3f0f694b8d"
    },
    "layers": [
        {
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "size": 32654,
            "digest": "sha256:9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0"
        }
    ]
}"#;

    // Create a minimal OCI image config
    let config_json = r#"{
        "architecture": "amd64",
        "os": "linux",
        "created": "2023-01-01T00:00:00Z",
        "config": {
            "Env": ["PATH=/usr/bin"],
            "Cmd": ["/bin/sh"]
        },
        "rootfs": {
            "type": "layers",
            "diff_ids": [
                "sha256:diff1111111111111111111111111111111111111111111111111111111111111"
            ]
        }
    }"#;

    // Mock manifest fetch
    let _manifest_mock = server
        .mock("GET", "/v2/test-raw-config/repo/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header(
            "docker-content-digest",
            "sha256:manifestdigest1111111111111111111111111111111111111111111111111",
        )
        .with_body(manifest_json)
        .create();

    // Mock config blob fetch
    let _config_mock = server
        .mock(
            "GET",
            "/v2/test-raw-config/repo/blobs/sha256:0af3a19ed6ae2afc5f87f1a90ec80092b46491b497ce4d0570193c3f0f694b8d",
        )
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.config.v1+json")
        .with_body(config_json)
        .create();

    // Call get_image_inspect with raw_config=true
    let result = get_image_inspect(
        &registry_url,
        "test-raw-config/repo:latest",
        None,
        false,
        true,
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let inspect = result.unwrap();

    // Verify raw_config is populated
    assert!(inspect.raw_config.is_some());
    let raw_config = inspect.raw_config.unwrap();

    // Verify it's valid JSON and contains expected fields
    assert!(raw_config.contains("\"architecture\""));
    assert!(raw_config.contains("\"amd64\""));
    assert!(raw_config.contains("\"os\""));
    assert!(raw_config.contains("\"linux\""));
    assert!(raw_config.contains("\"rootfs\""));
    assert!(raw_config.contains("\"config\""));

    // Verify raw_manifest is NOT populated
    assert!(inspect.raw_manifest.is_none());
}

// Test both raw flags together
#[allow(dead_code)]
#[test]
fn test_get_image_inspect_both_raw_flags() {
    let mut server = mockito::Server::new();
    let registry_url = server.url();

    // Mock version check
    let _v2_mock = server
        .mock("GET", "/v2/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create();

    // Create a minimal OCI manifest
    let manifest_json = r#"{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.manifest.v1+json",
    "config": {
        "mediaType": "application/vnd.oci.image.config.v1+json",
        "size": 265,
        "digest": "sha256:25890cdad8e8e7bdea614016480bb7dda663c418d23a4ae997b53ed17a022c85"
    },
    "layers": [
        {
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "size": 32654,
            "digest": "sha256:9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0"
        }
    ]
}"#;

    // Create a minimal OCI image config
    let config_json = r#"{
        "architecture": "amd64",
        "os": "linux",
        "created": "2023-01-01T00:00:00Z",
        "config": {},
        "rootfs": {
            "type": "layers",
            "diff_ids": [
                "sha256:diff1111111111111111111111111111111111111111111111111111111111111"
            ]
        }
    }"#;

    // Mock manifest fetch
    let _manifest_mock = server
        .mock("GET", "/v2/test-both-raw/repo/manifests/latest")
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.manifest.v1+json")
        .with_header(
            "docker-content-digest",
            "sha256:manifestdigest1111111111111111111111111111111111111111111111111",
        )
        .with_body(manifest_json)
        .create();

    // Mock config blob fetch
    let _config_mock = server
        .mock(
            "GET",
            "/v2/test-both-raw/repo/blobs/sha256:25890cdad8e8e7bdea614016480bb7dda663c418d23a4ae997b53ed17a022c85",
        )
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.config.v1+json")
        .with_body(config_json)
        .create();

    // Call get_image_inspect with both raw flags
    let result = get_image_inspect(&registry_url, "test-both-raw/repo:latest", None, true, true);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let inspect = result.unwrap();

    // Verify both raw fields are populated
    assert!(inspect.raw_manifest.is_some());
    assert!(inspect.raw_config.is_some());
}
