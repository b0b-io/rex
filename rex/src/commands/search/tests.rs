use super::*;

#[test]
fn test_search_results_serialization() {
    let results = SearchResults {
        query: "test".to_string(),
        images: ImageResults {
            total_results: 1,
            results: vec![ImageResult {
                name: "alpine".to_string(),
            }],
        },
        tags: TagResults {
            total_results: 1,
            results: vec![TagResult {
                image: "alpine".to_string(),
                tag: "latest".to_string(),
                reference: "alpine:latest".to_string(),
            }],
        },
    };

    let json = serde_json::to_string(&results).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("alpine"));
    assert!(json.contains("latest"));
}

#[test]
fn test_image_result_creation() {
    let result = ImageResult {
        name: "nginx".to_string(),
    };
    assert_eq!(result.name, "nginx");
}

#[test]
fn test_tag_result_creation() {
    let result = TagResult {
        image: "nginx".to_string(),
        tag: "1.21".to_string(),
        reference: "nginx:1.21".to_string(),
    };
    assert_eq!(result.image, "nginx");
    assert_eq!(result.tag, "1.21");
    assert_eq!(result.reference, "nginx:1.21");
}

#[test]
fn test_search_results_format_pretty_with_both() {
    let results = SearchResults {
        query: "test".to_string(),
        images: ImageResults {
            total_results: 2,
            results: vec![
                ImageResult {
                    name: "alpine".to_string(),
                },
                ImageResult {
                    name: "nginx".to_string(),
                },
            ],
        },
        tags: TagResults {
            total_results: 2,
            results: vec![
                TagResult {
                    image: "alpine".to_string(),
                    tag: "latest".to_string(),
                    reference: "alpine:latest".to_string(),
                },
                TagResult {
                    image: "nginx".to_string(),
                    tag: "1.21".to_string(),
                    reference: "nginx:1.21".to_string(),
                },
            ],
        },
    };

    let output = results.format_pretty();
    assert!(output.contains("Images:"));
    assert!(output.contains("alpine"));
    assert!(output.contains("nginx"));
    assert!(output.contains("Tags:"));
    assert!(output.contains("alpine:latest"));
    assert!(output.contains("nginx:1.21"));
}

#[test]
fn test_search_results_format_pretty_images_only() {
    let results = SearchResults {
        query: "test".to_string(),
        images: ImageResults {
            total_results: 1,
            results: vec![ImageResult {
                name: "alpine".to_string(),
            }],
        },
        tags: TagResults {
            total_results: 0,
            results: vec![],
        },
    };

    let output = results.format_pretty();
    assert!(output.contains("Images:"));
    assert!(output.contains("alpine"));
    assert!(!output.contains("Tags:"));
}

#[test]
fn test_search_results_format_pretty_tags_only() {
    let results = SearchResults {
        query: "test".to_string(),
        images: ImageResults {
            total_results: 0,
            results: vec![],
        },
        tags: TagResults {
            total_results: 1,
            results: vec![TagResult {
                image: "alpine".to_string(),
                tag: "latest".to_string(),
                reference: "alpine:latest".to_string(),
            }],
        },
    };

    let output = results.format_pretty();
    assert!(!output.contains("Images:"));
    assert!(output.contains("Tags:"));
    assert!(output.contains("alpine:latest"));
}

#[test]
fn test_search_results_format_pretty_no_results() {
    let results = SearchResults {
        query: "test".to_string(),
        images: ImageResults {
            total_results: 0,
            results: vec![],
        },
        tags: TagResults {
            total_results: 0,
            results: vec![],
        },
    };

    let output = results.format_pretty();
    assert!(output.contains("No results found"));
    assert!(!output.contains("Images:"));
    assert!(!output.contains("Tags:"));
}

#[test]
fn test_image_results_structure() {
    let results = ImageResults {
        total_results: 2,
        results: vec![
            ImageResult {
                name: "image1".to_string(),
            },
            ImageResult {
                name: "image2".to_string(),
            },
        ],
    };

    assert_eq!(results.total_results, 2);
    assert_eq!(results.results.len(), 2);
    assert_eq!(results.results[0].name, "image1");
    assert_eq!(results.results[1].name, "image2");
}

#[test]
fn test_tag_results_structure() {
    let results = TagResults {
        total_results: 2,
        results: vec![
            TagResult {
                image: "alpine".to_string(),
                tag: "3.14".to_string(),
                reference: "alpine:3.14".to_string(),
            },
            TagResult {
                image: "alpine".to_string(),
                tag: "latest".to_string(),
                reference: "alpine:latest".to_string(),
            },
        ],
    };

    assert_eq!(results.total_results, 2);
    assert_eq!(results.results.len(), 2);
    assert_eq!(results.results[0].reference, "alpine:3.14");
    assert_eq!(results.results[1].reference, "alpine:latest");
}
