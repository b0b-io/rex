use super::*;

const TEST_MANIFEST: &str = r#"{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.manifest.v1+json",
    "config": {
        "mediaType": "application/vnd.oci.image.config.v1+json",
        "size": 7023,
        "digest": "sha256:b5b2b2c507a0944348e0303114d8d93aaaa081732b86451d9bce1f432a537bc7"
    },
    "layers": [
        {
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "size": 32654,
            "digest": "sha256:9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0"
        }
    ]
}"#;

#[test]
fn test_image_manifest_deserialization() {
    let manifest: Result<ImageManifest, _> = serde_json::from_str(TEST_MANIFEST);
    assert!(manifest.is_ok());
    let manifest = manifest.unwrap();
    assert_eq!(manifest.schema_version(), 2);
    assert_eq!(
        manifest.config().digest().to_string(),
        "sha256:b5b2b2c507a0944348e0303114d8d93aaaa081732b86451d9bce1f432a537bc7"
    );
}

#[test]
fn test_types_are_accessible() {
    // This test doesn't need to do much. Its purpose is to fail compilation
    // if the types are not correctly re-exported.
    let _descriptor: Option<Descriptor> = None;
    let _image_config: Option<ImageConfiguration> = None;
    let _image_index: Option<ImageIndex> = None;
    let _platform: Option<Platform> = None;
}
