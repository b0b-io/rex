use super::Credentials;
use super::store::*;
use tempfile::tempdir;

#[test]
fn test_file_credential_store_new_creates_empty_store() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let store = FileCredentialStore::new(path.clone()).unwrap();
    assert!(store.list().unwrap().is_empty());
}

#[test]
fn test_file_credential_store_new_creates_parent_directory() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("subdir").join("credentials.toml");

    assert!(!path.parent().unwrap().exists());

    let _store = FileCredentialStore::new(path.clone()).unwrap();

    assert!(path.parent().unwrap().exists());
}

#[test]
fn test_file_credential_store_store_and_get_basic_credentials() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let mut store = FileCredentialStore::new(path.clone()).unwrap();

    let creds = Credentials::basic("testuser", "testpass");
    store.store("registry.example.com", &creds).unwrap();

    let retrieved = store.get("registry.example.com").unwrap();
    assert!(retrieved.is_some());

    if let Some(Credentials::Basic { username, password }) = retrieved {
        assert_eq!(username, "testuser");
        assert_eq!(password, "testpass");
    } else {
        panic!("Expected Basic credentials");
    }
}

#[test]
fn test_file_credential_store_get_nonexistent_returns_none() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let store = FileCredentialStore::new(path).unwrap();

    let retrieved = store.get("nonexistent.example.com").unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_file_credential_store_remove_credentials() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let mut store = FileCredentialStore::new(path).unwrap();

    let creds = Credentials::basic("testuser", "testpass");
    store.store("registry.example.com", &creds).unwrap();

    assert!(store.get("registry.example.com").unwrap().is_some());

    store.remove("registry.example.com").unwrap();

    assert!(store.get("registry.example.com").unwrap().is_none());
}

#[test]
fn test_file_credential_store_list_registries() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let mut store = FileCredentialStore::new(path).unwrap();

    let creds1 = Credentials::basic("user1", "pass1");
    let creds2 = Credentials::basic("user2", "pass2");

    store.store("registry1.example.com", &creds1).unwrap();
    store.store("registry2.example.com", &creds2).unwrap();

    let mut list = store.list().unwrap();
    list.sort();

    assert_eq!(list.len(), 2);
    assert_eq!(list[0], "registry1.example.com");
    assert_eq!(list[1], "registry2.example.com");
}

#[test]
fn test_file_credential_store_persists_across_instances() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    // Store credentials in first instance
    {
        let mut store = FileCredentialStore::new(path.clone()).unwrap();
        let creds = Credentials::basic("testuser", "testpass");
        store.store("registry.example.com", &creds).unwrap();
    }

    // Verify credentials persist in second instance
    {
        let store = FileCredentialStore::new(path.clone()).unwrap();
        let retrieved = store.get("registry.example.com").unwrap();
        assert!(retrieved.is_some());

        if let Some(Credentials::Basic { username, password }) = retrieved {
            assert_eq!(username, "testuser");
            assert_eq!(password, "testpass");
        } else {
            panic!("Expected Basic credentials");
        }
    }
}

#[test]
fn test_file_credential_store_overwrites_existing_credentials() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let mut store = FileCredentialStore::new(path).unwrap();

    // Store first set of credentials
    let creds1 = Credentials::basic("user1", "pass1");
    store.store("registry.example.com", &creds1).unwrap();

    // Overwrite with second set
    let creds2 = Credentials::basic("user2", "pass2");
    store.store("registry.example.com", &creds2).unwrap();

    // Verify only second set is present
    let retrieved = store.get("registry.example.com").unwrap();
    if let Some(Credentials::Basic { username, password }) = retrieved {
        assert_eq!(username, "user2");
        assert_eq!(password, "pass2");
    } else {
        panic!("Expected Basic credentials");
    }
}

#[test]
fn test_file_credential_store_store_anonymous_fails() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let mut store = FileCredentialStore::new(path).unwrap();

    let creds = Credentials::Anonymous;
    let result = store.store("registry.example.com", &creds);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Cannot store anonymous credentials")
    );
}

#[test]
fn test_file_credential_store_store_bearer_fails() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let mut store = FileCredentialStore::new(path).unwrap();

    let creds = Credentials::Bearer {
        token: "test_token".to_string(),
    };
    let result = store.store("registry.example.com", &creds);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Bearer token storage not yet supported")
    );
}

#[test]
#[cfg(unix)]
fn test_file_credential_store_sets_restrictive_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let mut store = FileCredentialStore::new(path.clone()).unwrap();

    let creds = Credentials::basic("testuser", "testpass");
    store.store("registry.example.com", &creds).unwrap();

    // Check file permissions are 0600 (user read/write only)
    let metadata = std::fs::metadata(&path).unwrap();
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // On Unix, mode includes file type bits, so we mask to get just permission bits
    let permission_bits = mode & 0o777;
    assert_eq!(permission_bits, 0o600);
}

#[test]
fn test_file_credential_store_handles_multiple_registries() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let mut store = FileCredentialStore::new(path).unwrap();

    // Store credentials for multiple registries
    store
        .store(
            "registry1.example.com",
            &Credentials::basic("user1", "pass1"),
        )
        .unwrap();
    store
        .store(
            "registry2.example.com",
            &Credentials::basic("user2", "pass2"),
        )
        .unwrap();
    store
        .store(
            "registry3.example.com",
            &Credentials::basic("user3", "pass3"),
        )
        .unwrap();

    // Verify each registry has its own credentials
    let creds1 = store.get("registry1.example.com").unwrap().unwrap();
    let creds2 = store.get("registry2.example.com").unwrap().unwrap();
    let creds3 = store.get("registry3.example.com").unwrap().unwrap();

    if let Credentials::Basic { username, .. } = creds1 {
        assert_eq!(username, "user1");
    }
    if let Credentials::Basic { username, .. } = creds2 {
        assert_eq!(username, "user2");
    }
    if let Credentials::Basic { username, .. } = creds3 {
        assert_eq!(username, "user3");
    }
}

#[test]
fn test_file_credential_store_list_empty_after_remove_all() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().join("credentials.toml");

    let mut store = FileCredentialStore::new(path).unwrap();

    // Store and then remove credentials
    store
        .store(
            "registry1.example.com",
            &Credentials::basic("user1", "pass1"),
        )
        .unwrap();
    store
        .store(
            "registry2.example.com",
            &Credentials::basic("user2", "pass2"),
        )
        .unwrap();

    assert_eq!(store.list().unwrap().len(), 2);

    store.remove("registry1.example.com").unwrap();
    store.remove("registry2.example.com").unwrap();

    assert!(store.list().unwrap().is_empty());
}
