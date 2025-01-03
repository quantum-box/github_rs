use github_rs::client::GitHubClient;

/// プルリクエスト作成のサンプルコード
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // GitHubクライアントの初期化
    let token = std::env::var("GITHUB_TOKEN")?;
    let client = GitHubClient::new(token);

    // リポジトリ情報
    let owner = "owner";
    let repo = "repo";
    let base_branch = "main";
    let head_branch = "feature";
    let pr_title = "新機能の追加";
    let pr_body = "このプルリクエストは新機能を追加します。\n\n変更内容:\n- 機能A\n- 機能B";

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
