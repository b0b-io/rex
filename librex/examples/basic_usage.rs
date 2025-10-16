//! Basic usage example for the Rex library.
//!
//! This example demonstrates the high-level API for interacting with
//! OCI registries.
//!
//! Run with: cargo run --example basic_usage

use librex::Rex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Rex Library - Basic Usage Example\n");

    // Connect to a local registry
    let mut rex = Rex::connect("http://localhost:5000").await?;
    println!("✓ Connected to registry: {}\n", rex.registry_url());

    // Check if registry is accessible
    match rex.check().await {
        Ok(_) => println!("✓ Registry is accessible and supports OCI Distribution Spec\n"),
        Err(e) => {
            eprintln!("✗ Failed to connect: {}", e);
            eprintln!("  Make sure a registry is running at http://localhost:5000");
            eprintln!(
                "  You can start one with: docker run -d -p 5000:5000 ghcr.io/project-zot/zot-linux-amd64:latest"
            );
            return Ok(());
        }
    }

    // List all repositories
    println!("Fetching repositories...");
    match rex.list_repositories().await {
        Ok(repos) => {
            println!("✓ Found {} repositories:\n", repos.len());
            for repo in repos.iter().take(10) {
                println!("  - {}", repo);
            }
            if repos.len() > 10 {
                println!("  ... and {} more", repos.len() - 10);
            }
            println!();

            // If we have repositories, list tags for the first one
            if let Some(first_repo) = repos.first() {
                println!("Fetching tags for '{}'...", first_repo);
                match rex.list_tags(first_repo).await {
                    Ok(tags) => {
                        println!("✓ Found {} tags:\n", tags.len());
                        for tag in tags.iter().take(5) {
                            println!("  - {}:{}", first_repo, tag);
                        }
                        if tags.len() > 5 {
                            println!("  ... and {} more", tags.len() - 5);
                        }
                        println!();
                    }
                    Err(e) => println!("✗ Failed to fetch tags: {}\n", e),
                }

                // Search for repositories
                println!("Searching for repositories matching 'alp'...");
                match rex.search_repositories("alp").await {
                    Ok(results) => {
                        if results.is_empty() {
                            println!("  No matches found\n");
                        } else {
                            println!("✓ Found {} matches:\n", results.len());
                            for result in results.iter().take(5) {
                                println!("  - {} (score: {})", result.value, result.score);
                            }
                            println!();
                        }
                    }
                    Err(e) => println!("✗ Search failed: {}\n", e),
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to list repositories: {}", e);
            println!("  The registry might be empty or require authentication\n");
        }
    }

    println!("Example completed!");
    Ok(())
}
