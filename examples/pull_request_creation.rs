use github::client::GitHubClient;

/// プルリクエスト作成のサンプルコード
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // GitHubクライアントの初期化
    let token = std::env::var("GITHUB_TOKEN")?;
    let client = GitHubClient::new(token);

    // リポジトリ情報
    let owner = "quantum-box";
    let repo = "github_rs";
    let base_branch = "main";
    let head_branch = format!("test-branch-{}", chrono::Utc::now().timestamp());
    let pr_title = "テスト用プルリクエスト";
    let pr_body = "このプルリクエストはexampleのテスト実行により作成されました。\n\n自動テスト:\n- プルリクエスト作成機能のテスト\n- APIの動作確認";

    println!("プルリクエストを作成中...");
    match client
        .create_pull_request(
            owner,
            repo,
            base_branch,
            head_branch,
            pr_title,
            pr_body,
        )
        .await
    {
        Ok(()) => {
            println!("✓ プルリクエストの作成に成功しました");
            println!("  - ベースブランチ: {}", base_branch);
            println!("  - ヘッドブランチ: {}", head_branch);
            println!("  - タイトル: {}", pr_title);
        }
        Err(e) => {
            println!("✗ プルリクエストの作成に失敗しました: {}", e);
            if let Some(status) = e.status() {
                match status {
                    reqwest::StatusCode::FORBIDDEN => {
                        println!("トークンの権限が不足しているか、無効なトークンです");
                    }
                    reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
                        println!("指定されたブランチが存在しないか、既にプルリクエストが存在します");
                    }
                    _ => println!("APIエラー: ステータスコード {}", status),
                }
            }
        }
    }

    Ok(())
}
