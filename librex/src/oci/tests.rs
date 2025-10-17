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
    let manifest: std::result::Result<ImageManifest, _> = serde_json::from_str(TEST_MANIFEST);
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

const TEST_INDEX: &str = r#"{
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

#[test]
fn test_manifest_or_index_from_manifest() {
    let result = ManifestOrIndex::from_bytes(TEST_MANIFEST.as_bytes());
    assert!(result.is_ok());
    let manifest_or_index = result.unwrap();
    assert!(manifest_or_index.is_manifest());
    assert!(!manifest_or_index.is_index());
    assert!(manifest_or_index.as_manifest().is_some());
    assert!(manifest_or_index.as_index().is_none());
}

#[test]
fn test_manifest_or_index_from_index() {
    let result = ManifestOrIndex::from_bytes(TEST_INDEX.as_bytes());
    assert!(result.is_ok());
    let manifest_or_index = result.unwrap();
    assert!(manifest_or_index.is_index());
    assert!(!manifest_or_index.is_manifest());
    assert!(manifest_or_index.as_index().is_some());
    assert!(manifest_or_index.as_manifest().is_none());
}

#[test]
fn test_manifest_or_index_platforms() {
    let manifest_or_index = ManifestOrIndex::from_bytes(TEST_INDEX.as_bytes()).unwrap();
    let platforms = manifest_or_index.platforms();
    assert_eq!(platforms.len(), 2);

    let (platform1, _) = platforms[0];
    assert_eq!(platform1.architecture().to_string(), "amd64");
    assert_eq!(platform1.os().to_string(), "linux");

    let (platform2, _) = platforms[1];
    assert_eq!(platform2.architecture().to_string(), "arm64");
    assert_eq!(platform2.os().to_string(), "linux");
}

#[test]
fn test_manifest_or_index_find_platform() {
    let manifest_or_index = ManifestOrIndex::from_bytes(TEST_INDEX.as_bytes()).unwrap();

    let amd64 = manifest_or_index.find_platform("linux", "amd64");
    assert!(amd64.is_some());
    assert_eq!(
        amd64.unwrap().digest().to_string(),
        "sha256:aaaa1234567890abcdef1234567890abcdef1234567890abcdef123456789012"
    );

    let arm64 = manifest_or_index.find_platform("linux", "arm64");
    assert!(arm64.is_some());
    assert_eq!(
        arm64.unwrap().digest().to_string(),
        "sha256:bbbb1234567890abcdef1234567890abcdef1234567890abcdef123456789012"
    );

    let not_found = manifest_or_index.find_platform("windows", "amd64");
    assert!(not_found.is_none());
}

#[test]
fn test_manifest_or_index_into_manifest() {
    let manifest_or_index = ManifestOrIndex::from_bytes(TEST_MANIFEST.as_bytes()).unwrap();
    let manifest = manifest_or_index.into_manifest();
    assert!(manifest.is_some());
}

#[test]
fn test_manifest_or_index_into_index() {
    let manifest_or_index = ManifestOrIndex::from_bytes(TEST_INDEX.as_bytes()).unwrap();
    let index = manifest_or_index.into_index();
    assert!(index.is_some());
}
