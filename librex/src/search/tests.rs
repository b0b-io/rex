use super::*;
use std::collections::HashMap;

#[test]
fn test_empty_query_returns_all() {
    let targets = vec![
        "alpine".to_string(),
        "ubuntu".to_string(),
        "nginx".to_string(),
    ];
    let results = fuzzy_search("", &targets, CaseMatching::Ignore);

    assert_eq!(results.len(), 3);
    // All should have score 0 for empty query
    assert!(results.iter().all(|r| r.score == 0));
}

#[test]
fn test_exact_match() {
    let targets = vec![
        "alpine".to_string(),
        "ubuntu".to_string(),
        "nginx".to_string(),
    ];
    let results = fuzzy_search("alpine", &targets, CaseMatching::Ignore);

    assert!(!results.is_empty());
    assert_eq!(results[0].value, "alpine");
    // Exact match should have highest score
    assert!(results[0].score > 0);
}

#[test]
fn test_prefix_match() {
    let targets = vec![
        "alpine".to_string(),
        "ubuntu".to_string(),
        "nginx".to_string(),
    ];
    let results = fuzzy_search("alp", &targets, CaseMatching::Ignore);

    assert!(!results.is_empty());
    assert_eq!(results[0].value, "alpine");
}

#[test]
fn test_fuzzy_match() {
    let targets = vec![
        "alpine".to_string(),
        "ubuntu".to_string(),
        "nginx".to_string(),
    ];
    let results = fuzzy_search("apn", &targets, CaseMatching::Ignore);

    // "apn" should match "alpine" (a-p-i-n-e contains a, p, n in order)
    assert!(results.iter().any(|r| r.value == "alpine"));
}

#[test]
fn test_no_match() {
    let targets = vec![
        "alpine".to_string(),
        "ubuntu".to_string(),
        "nginx".to_string(),
    ];
    let results = fuzzy_search("xyz", &targets, CaseMatching::Ignore);

    // "xyz" shouldn't match any of these
    assert!(results.is_empty());
}

#[test]
fn test_case_insensitive_match() {
    let targets = vec![
        "Alpine".to_string(),
        "Ubuntu".to_string(),
        "NGINX".to_string(),
    ];
    let results = fuzzy_search("alpine", &targets, CaseMatching::Ignore);

    assert!(!results.is_empty());
    assert_eq!(results[0].value, "Alpine");
}

#[test]
fn test_case_smart_match() {
    let targets = vec!["Alpine".to_string(), "alpine".to_string()];

    // Lowercase query should match both (case-insensitive)
    let results1 = fuzzy_search("alpine", &targets, CaseMatching::Smart);
    assert_eq!(results1.len(), 2);

    // Uppercase query should prefer exact case match
    let results2 = fuzzy_search("Alpine", &targets, CaseMatching::Smart);
    assert!(!results2.is_empty());
}

#[test]
fn test_results_sorted_by_score() {
    let targets = vec![
        "alpine".to_string(),
        "application".to_string(),
        "app".to_string(),
    ];
    let results = fuzzy_search("app", &targets, CaseMatching::Ignore);

    assert!(!results.is_empty());
    // Exact match "app" should score highest
    assert_eq!(results[0].value, "app");

    // Verify scores are in descending order
    for i in 1..results.len() {
        assert!(results[i - 1].score >= results[i].score);
    }
}

#[test]
fn test_search_repositories() {
    let repos = vec![
        "alpine".to_string(),
        "ubuntu".to_string(),
        "nginx".to_string(),
        "postgres".to_string(),
    ];

    let results = search_repositories("alp", &repos);
    assert!(!results.is_empty());
    assert_eq!(results[0].value, "alpine");
}

#[test]
fn test_search_tags() {
    let tags = vec![
        "latest".to_string(),
        "3.19".to_string(),
        "3.18".to_string(),
        "stable".to_string(),
    ];

    let results = search_tags("lat", &tags);
    assert!(!results.is_empty());
    assert_eq!(results[0].value, "latest");
}

#[test]
fn test_search_tags_with_version() {
    let tags = vec![
        "latest".to_string(),
        "3.19".to_string(),
        "3.18".to_string(),
        "stable".to_string(),
    ];

    let results = search_tags("3.19", &tags);
    assert!(!results.is_empty());
    assert_eq!(results[0].value, "3.19");
}

#[test]
fn test_search_images_repo_only() {
    let repos = vec!["alpine".to_string(), "ubuntu".to_string()];
    let mut tags_map = HashMap::new();
    tags_map.insert(
        "alpine".to_string(),
        vec!["latest".to_string(), "3.19".to_string()],
    );
    tags_map.insert(
        "ubuntu".to_string(),
        vec!["latest".to_string(), "22.04".to_string()],
    );

    let results = search_images("alp", &repos, &tags_map);

    // Should return alpine with all its tags
    assert!(!results.is_empty());
    assert!(results.iter().any(|r| r.value == "alpine:latest"));
    assert!(results.iter().any(|r| r.value == "alpine:3.19"));
    // Should not include ubuntu
    assert!(!results.iter().any(|r| r.value.starts_with("ubuntu")));
}

#[test]
fn test_search_images_with_tag() {
    let repos = vec!["alpine".to_string(), "ubuntu".to_string()];
    let mut tags_map = HashMap::new();
    tags_map.insert(
        "alpine".to_string(),
        vec!["latest".to_string(), "3.19".to_string()],
    );
    tags_map.insert(
        "ubuntu".to_string(),
        vec!["latest".to_string(), "22.04".to_string()],
    );

    let results = search_images("alp:lat", &repos, &tags_map);

    // Should return only alpine:latest
    assert!(!results.is_empty());
    assert_eq!(results[0].value, "alpine:latest");
    // Should not include alpine:3.19
    assert!(!results.iter().any(|r| r.value == "alpine:3.19"));
}

#[test]
fn test_search_images_with_version_tag() {
    let repos = vec!["alpine".to_string(), "ubuntu".to_string()];
    let mut tags_map = HashMap::new();
    tags_map.insert(
        "alpine".to_string(),
        vec!["latest".to_string(), "3.19".to_string(), "3.18".to_string()],
    );
    tags_map.insert(
        "ubuntu".to_string(),
        vec!["latest".to_string(), "22.04".to_string()],
    );

    let results = search_images("alp:3.19", &repos, &tags_map);

    // Should return alpine:3.19
    assert!(!results.is_empty());
    assert_eq!(results[0].value, "alpine:3.19");
}

#[test]
fn test_search_images_fuzzy_both() {
    let repos = vec![
        "alpine".to_string(),
        "ubuntu".to_string(),
        "debian".to_string(),
    ];
    let mut tags_map = HashMap::new();
    tags_map.insert(
        "alpine".to_string(),
        vec!["latest".to_string(), "stable".to_string()],
    );
    tags_map.insert(
        "ubuntu".to_string(),
        vec!["latest".to_string(), "22.04".to_string()],
    );
    tags_map.insert(
        "debian".to_string(),
        vec!["latest".to_string(), "stable".to_string()],
    );

    let results = search_images("alp:stb", &repos, &tags_map);

    // Should match alpine (alp) with stable (stb)
    assert!(!results.is_empty());
    assert_eq!(results[0].value, "alpine:stable");
}

#[test]
fn test_search_images_no_tags_for_repo() {
    let repos = vec!["alpine".to_string(), "ubuntu".to_string()];
    let mut tags_map = HashMap::new();
    tags_map.insert("alpine".to_string(), vec!["latest".to_string()]);
    // ubuntu has no tags in the map

    let results = search_images("ubuntu", &repos, &tags_map);

    // Should return empty because ubuntu has no tags
    assert!(results.is_empty());
}

#[test]
fn test_search_images_empty_query() {
    let repos = vec!["alpine".to_string(), "ubuntu".to_string()];
    let mut tags_map = HashMap::new();
    tags_map.insert("alpine".to_string(), vec!["latest".to_string()]);
    tags_map.insert("ubuntu".to_string(), vec!["latest".to_string()]);

    let results = search_images("", &repos, &tags_map);

    // Empty query should return all images
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_result_ordering() {
    let targets = vec![
        "test".to_string(),
        "testing".to_string(),
        "latest".to_string(),
    ];

    let results = fuzzy_search("test", &targets, CaseMatching::Ignore);

    // Exact match should be first
    assert_eq!(results[0].value, "test");

    // For equal scores, alphabetical order
    if results.len() > 1 && results[0].score == results[1].score {
        assert!(results[0].value <= results[1].value);
    }
}

#[test]
fn test_multiple_matches_same_score() {
    let targets = vec!["abc".to_string(), "abd".to_string(), "abe".to_string()];

    let results = fuzzy_search("ab", &targets, CaseMatching::Ignore);

    // All should match with same score (prefix match)
    assert_eq!(results.len(), 3);

    // Should be in alphabetical order
    assert_eq!(results[0].value, "abc");
    assert_eq!(results[1].value, "abd");
    assert_eq!(results[2].value, "abe");
}

#[test]
fn test_search_with_special_characters() {
    let targets = vec![
        "my-app".to_string(),
        "my_app".to_string(),
        "myapp".to_string(),
    ];

    let results = fuzzy_search("my-app", &targets, CaseMatching::Ignore);

    // Should match "my-app" exactly
    assert!(!results.is_empty());
    assert_eq!(results[0].value, "my-app");
}

#[test]
fn test_search_with_numbers() {
    let tags = vec![
        "v1.2.3".to_string(),
        "v1.2.4".to_string(),
        "v2.0.0".to_string(),
    ];

    let results = search_tags("1.2.3", &tags);

    // Should match v1.2.3
    assert!(!results.is_empty());
    assert_eq!(results[0].value, "v1.2.3");
}
