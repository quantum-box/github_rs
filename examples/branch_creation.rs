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

    println!("Testing Branch Creation API...\n");

    let owner = "quantum-box";
    let repo = "github_rs";
    let base_branch = "main";
    let new_branch = format!("test-branch-{}", chrono::Utc::now().timestamp());

    println!("Getting SHA of {} branch...", base_branch);
    match client.get_base_branch_sha(owner, repo, base_branch).await {
        Ok(base_sha) => {
            println!("✓ Got base SHA: {:.8}...", base_sha);
            
            println!("Creating new branch: {}...", new_branch);
            match client.create_branch(owner, repo, &new_branch, &base_sha).await {
                Ok(()) => {
                    println!("✓ Successfully created branch: {}", new_branch);
                }
                Err(e) => {
                    println!("✗ Failed to create branch: {}", e);
                    if let Some(status) = e.status() {
                        if status == reqwest::StatusCode::FORBIDDEN {
                            println!("This might be due to invalid token or insufficient permissions");
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to get base SHA: {}", e);
            if let Some(status) = e.status() {
                if status == reqwest::StatusCode::FORBIDDEN {
                    println!("This might be due to invalid token or insufficient permissions");
                }
            }
        }
    }

    Ok(())
}
