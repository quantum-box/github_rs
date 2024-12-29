use github::client::GitHubClient;
use github::auth::AuthToken;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth_token = AuthToken::from_env()?;
    let client = GitHubClient::new(auth_token.as_str().to_string());

    println!("Testing GitHub API Client...\n");

    // Test 1: Get user information
    println!("Test 1: Getting user information...");
    let response = client.get("/user").await?;
    if response.status().is_success() {
        let user_info: serde_json::Value = response.json().await?;
        println!("✓ Successfully retrieved user info:");
        println!("  Login: {}", user_info["login"]);
        println!("  Name: {}", user_info["name"]);
    } else {
        println!("✗ Failed to get user info: {}", response.status());
    }

    // Test 2: List repositories
    println!("\nTest 2: Listing repositories...");
    let response = client.get_user_repos().await?;
    if response.status().is_success() {
        let repos: Vec<serde_json::Value> = response.json().await?;
        println!("✓ Successfully retrieved repositories:");
        for repo in repos.iter().take(5) {
            println!("  - {} ({})", repo["name"], repo["html_url"]);
        }
        if repos.len() > 5 {
            println!("  ... and {} more", repos.len() - 5);
        }
    } else {
        println!("✗ Failed to list repos: {}", response.status());
    }

    // Test 3: Test error handling (404)
    println!("\nTest 3: Testing error handling (404)...");
    let response = client.get("/non_existent_endpoint").await?;
    println!("✓ Got expected status code: {}", response.status());

    println!("\nAll tests completed!");
    Ok(())
}
