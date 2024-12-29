use github_rs::client::GitHubClient;

/// コミット作成のサンプルコード
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // GitHubクライアントの初期化
    let token = std::env::var("GITHUB_TOKEN")?;
    let client = GitHubClient::new(token);

    // リポジトリ情報
    let owner = "owner";
    let repo = "repo";
    let branch = "main";
    let file_path = "example/test.txt";
    let file_content = "Hello, World!";
    let commit_message = "Add test.txt";

    // 1. ベースブランチの最新コミットSHAを取得
    let base_commit_sha = client.get_base_branch_sha(owner, repo, branch).await?;

    // 2. 最新コミットのツリーSHAを取得
    let base_tree_sha = client
        .get_latest_tree_sha(owner, repo, &base_commit_sha)
        .await?;

    // 3. ファイル内容のBLOBを作成
    let blob_sha = client.create_blob(owner, repo, file_content).await?;

    // 4. 新しいツリーを作成
    let new_tree_sha = client
        .create_tree(owner, repo, &base_tree_sha, file_path, &blob_sha)
        .await?;

    // 5. 新しいコミットを作成
    let new_commit_sha = client
        .create_commit(owner, repo, commit_message, &new_tree_sha, &base_commit_sha)
        .await?;

    // 6. ブランチの先端を更新
    client
        .update_branch_reference(owner, repo, branch, &new_commit_sha)
        .await?;

    println!("Successfully created commit: {}", new_commit_sha);
    Ok(())
}
