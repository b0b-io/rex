use super::*;

#[test]
fn test_credentials_anonymous() {
    let creds = Credentials::anonymous();
    assert_eq!(creds, Credentials::Anonymous);
    assert_eq!(creds.to_header_value(), None);
}

#[test]
fn test_credentials_basic() {
    let creds = Credentials::basic("testuser", "testpass");
    match &creds {
        Credentials::Basic { username, password } => {
            assert_eq!(username, "testuser");
            assert_eq!(password, "testpass");
        }
        _ => panic!("Expected Basic credentials"),
    }

    let header = creds.to_header_value().unwrap();
    assert!(header.starts_with("Basic "));
}

#[test]
fn test_credentials_bearer() {
    let creds = Credentials::bearer("my_token");
    match &creds {
        Credentials::Bearer { token } => {
            assert_eq!(token, "my_token");
        }
        _ => panic!("Expected Bearer credentials"),
    }

    let header = creds.to_header_value().unwrap();
    assert_eq!(header, "Bearer my_token");
}

#[test]
fn test_auth_challenge_parse_bearer() {
    let header = r#"Bearer realm="https://auth.example.com/token",service="registry.example.com",scope="repository:alpine:pull""#;

    let challenge = AuthChallenge::parse(header).unwrap();
    assert_eq!(challenge.scheme, "Bearer");
    assert_eq!(challenge.realm, "https://auth.example.com/token");
    assert_eq!(challenge.service, Some("registry.example.com".to_string()));
    assert_eq!(challenge.scope, Some("repository:alpine:pull".to_string()));
}

#[test]
fn test_auth_challenge_parse_without_service() {
    let header = r#"Bearer realm="https://auth.example.com/token",scope="repository:alpine:pull""#;

    let challenge = AuthChallenge::parse(header).unwrap();
    assert_eq!(challenge.scheme, "Bearer");
    assert_eq!(challenge.realm, "https://auth.example.com/token");
    assert_eq!(challenge.service, None);
    assert_eq!(challenge.scope, Some("repository:alpine:pull".to_string()));
}

#[test]
fn test_auth_challenge_parse_without_scope() {
    let header = r#"Bearer realm="https://auth.example.com/token",service="registry""#;

    let challenge = AuthChallenge::parse(header).unwrap();
    assert_eq!(challenge.scheme, "Bearer");
    assert_eq!(challenge.realm, "https://auth.example.com/token");
    assert_eq!(challenge.service, Some("registry".to_string()));
    assert_eq!(challenge.scope, None);
}

#[test]
fn test_auth_challenge_parse_basic() {
    let header = r#"Basic realm="Registry Access""#;

    let challenge = AuthChallenge::parse(header).unwrap();
    assert_eq!(challenge.scheme, "Basic");
    assert_eq!(challenge.realm, "Registry Access");
    assert_eq!(challenge.service, None);
    assert_eq!(challenge.scope, None);
}

#[test]
fn test_auth_challenge_parse_missing_realm() {
    let header = r#"Bearer service="registry""#;

    let result = AuthChallenge::parse(header);
    assert!(result.is_err());
}

#[test]
fn test_auth_challenge_parse_invalid_format() {
    let header = "InvalidHeader";

    let result = AuthChallenge::parse(header);
    assert!(result.is_err());
}
