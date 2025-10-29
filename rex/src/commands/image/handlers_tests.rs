use super::*;

// Note: These tests use mockito to test get_image_inspect end-to-end with mock HTTP responses.
// The tests verify that the function correctly calls the registry, parses responses, and
// handles both success and error cases.

#[tokio::test]
async fn test_get_image_inspect_invalid_reference() {
    let server = mockito::Server::new_async().await;
    let registry_url = server.url();

    // Call get_image_inspect with invalid reference
    let result = get_image_inspect(&registry_url, "").await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Invalid image reference"));
}

#[tokio::test]
async fn test_get_image_inspect_manifest_not_found() {
    let mut server = mockito::Server::new_async().await;
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
    let result = get_image_inspect(&registry_url, "test/repo:nonexistent").await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Failed to fetch manifest"));
}

// TODO: Full integration test with real OCI structures
// Currently commented out due to oci-spec validation being very strict about JSON format
// The success case is tested in integration tests with real registries
#[allow(dead_code)]
#[tokio::test]
async fn test_get_image_inspect_single_platform() {
    let mut server = mockito::Server::new_async().await;
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
        .mock("GET", "/v2/test/repo/manifests/latest")
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
            "/v2/test/repo/blobs/sha256:05d6eacdcaf34accb9bfcc28ce285c4aee9844550f36890488468f9bcceebd76",
        )
        .with_status(200)
        .with_header("content-type", "application/vnd.oci.image.config.v1+json")
        .with_body(config_json)
        .create();

    // Call get_image_inspect
    let result = get_image_inspect(&registry_url, "test/repo:latest").await;

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let inspect = result.unwrap();

    // Verify basic fields
    assert_eq!(inspect.reference, "test/repo:latest");
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

// TODO: Add test for multi-platform image index error case
// Currently commented out due to oci-spec validation complexity
// The error case is tested in integration tests with real registries
