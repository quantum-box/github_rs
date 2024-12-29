use github::auth::AuthToken;
use github::client::GitHubClient;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    github::init_tracing();
    tracing::info!("Initializing GitHub API client");
    let auth_token = AuthToken::from_env()?;
    let client = GitHubClient::new(auth_token.as_str().to_string());
    tracing::info!("GitHub API client initialized");

    println!("Testing GitHub API Client...\n");

    // Test 1: Get user information
    println!("Test 1: Getting user information...");
    match client.get("/user").await {
        Ok(response) => {
            let user_info: serde_json::Value = response.json().await?;
            println!("✓ Successfully retrieved user info:");
            println!("  Login: {}", user_info["login"]);
            println!("  Name: {}", user_info["name"]);
        }
        Err(e) => {
            println!("✗ Failed to get user info: {}", e);
            if let Some(status) = e.status() {
                if status == reqwest::StatusCode::FORBIDDEN {
                    println!("This might be due to invalid token or insufficient permissions");
                }
            }
        }
    }

    // Test 2: List repositories
    println!("\nTest 2: Listing repositories...");
    match client.get_user_repos().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                let repos: Vec<serde_json::Value> = response.json().await?;
                println!("✓ Successfully retrieved repositories:");
                for repo in repos.iter().take(5) {
                    println!("  - {} ({})", repo["name"], repo["html_url"]);
                }
                if repos.len() > 5 {
                    println!("  ... and {} more", repos.len() - 5);
                }
            } else {
                println!("✗ Failed to list repos: {}", status);
                if status == reqwest::StatusCode::FORBIDDEN {
                    println!("This might be due to invalid token or insufficient permissions");
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to list repos: {}", e);
            if let Some(status) = e.status() {
                if status == reqwest::StatusCode::FORBIDDEN {
                    println!("This might be due to invalid token or insufficient permissions");
                }
            }
        }
    }

    // Test 3: Create a new branch
    println!("\nTest 3: Creating a new branch...");
    let owner = "quantum-box";  // Replace with actual owner if different
    let repo = "github_rs";     // Replace with actual repo if different
    let base_branch = "main";
    let new_branch = format!("test-branch-{}", chrono::Utc::now().timestamp());

    println!("  Getting SHA of {} branch...", base_branch);
    match client.get_base_branch_sha(owner, repo, base_branch).await {
        Ok(base_sha) => {
            println!("  ✓ Got base SHA: {:.8}...", base_sha);
            
            println!("  Creating new branch: {}...", new_branch);
            match client.create_branch(owner, repo, &new_branch, &base_sha).await {
                Ok(()) => {
                    println!("  ✓ Successfully created branch: {}", new_branch);
                }
                Err(e) => {
                    println!("  ✗ Failed to create branch: {}", e);
                    if let Some(status) = e.status() {
                        if status == reqwest::StatusCode::FORBIDDEN {
                            println!("  This might be due to invalid token or insufficient permissions");
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("  ✗ Failed to get base SHA: {}", e);
            if let Some(status) = e.status() {
                if status == reqwest::StatusCode::FORBIDDEN {
                    println!("  This might be due to invalid token or insufficient permissions");
                }
            }
        }
    }

    // Test 4: Test error handling (404)
    println!("\nTest 4: Testing error handling (404)...");
    let response = client.get("/non_existent_endpoint").await?;
    println!("✓ Got expected status code: {}", response.status());

    println!("\nAll tests completed!");
    Ok(())
}
