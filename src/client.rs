use crate::auth::{build_auth_headers, AuthToken};
use reqwest::{Client, Response};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    #[error("Request failed with status {status}: {message}")]
    ApiError {
        status: reqwest::StatusCode,
        message: String,
    },
}

impl GitHubError {
    pub fn status(&self) -> Option<reqwest::StatusCode> {
        match self {
            GitHubError::RequestError(e) => e.status(),
            GitHubError::ApiError { status, .. } => Some(*status),
            _ => None,
        }
    }
}

pub struct GitHubClient {
    http: Client,
    token: AuthToken,
    base_url: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("github-rs-client"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http: client,
            token: AuthToken::new(token),
            base_url: "https://api.github.com".to_string(),
        }
    }

    pub async fn get(&self, path: &str) -> reqwest::Result<Response> {
        use tracing::{debug, info, warn};

        let url = format!("{}{}", self.base_url, path);
        info!(target: "github_client", method = "GET", %url, "Making API request");

        let headers = build_auth_headers(self.token.as_str());
        debug!(target: "github_client", ?headers, "Request headers prepared");

        let response = self.http.get(url).headers(headers).send().await?;
        let status = response.status();

        if !status.is_success() {
            warn!(
                target: "github_client",
                %status,
                endpoint = %path,
                "Request failed"
            );
        } else {
            info!(
                target: "github_client",
                %status,
                endpoint = %path,
                "Request successful"
            );
        }

        response.error_for_status()
    }

    pub async fn post<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> reqwest::Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let headers = build_auth_headers(self.token.as_str());
        self.http.post(url).headers(headers).json(body).send().await
    }

    pub async fn patch<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> reqwest::Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let headers = build_auth_headers(self.token.as_str());
        self.http
            .patch(url)
            .headers(headers)
            .json(body)
            .send()
            .await
    }

    // Example API method using the generic request methods
    pub async fn get_user_repos(&self) -> reqwest::Result<Response> {
        self.get("/user/repos").await
    }

    /// Get the latest commit SHA of a base branch
    pub async fn get_base_branch_sha(
        &self,
        owner: &str,
        repo: &str,
        base_branch: &str,
    ) -> Result<String, GitHubError> {
        let path = format!("/repos/{}/{}/git/ref/heads/{}", owner, repo, base_branch);
        let response = self.get(&path).await?;
        let status = response.status();

        if !status.is_success() {
            let error_json: Value = response.json().await?;
            let message = error_json["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(GitHubError::ApiError { status, message });
        }

        let json: Value = response.json().await?;

        // Extract the SHA from the response JSON
        json.get("object")
            .and_then(|obj| obj.get("sha"))
            .and_then(|sha| sha.as_str())
            .map(String::from)
            .ok_or_else(|| {
                GitHubError::ParseError("Failed to extract SHA from response".to_string())
            })
    }

    /// Create a new branch using a base SHA
    pub async fn create_branch(
        &self,
        owner: &str,
        repo: &str,
        new_branch_name: &str,
        base_sha: &str,
    ) -> Result<(), GitHubError> {
        let path = format!("/repos/{}/{}/git/refs", owner, repo);
        let body = serde_json::json!({
            "ref": format!("refs/heads/{}", new_branch_name),
            "sha": base_sha
        });

        let response = self.post(&path, &body).await?;
        let status = response.status();

        if !status.is_success() {
            let error_json: Value = response.json().await?;
            let message = error_json["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(GitHubError::ApiError { status, message });
        }
        Ok(())
    }

    /// 最新のコミットのツリーSHAを取得する
    pub async fn get_latest_tree_sha(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<String, GitHubError> {
        let path = format!("/repos/{}/{}/git/commits/{}", owner, repo, commit_sha);
        let response = self.get(&path).await?;
        let status = response.status();

        if !status.is_success() {
            let error_json: Value = response.json().await?;
            let message = error_json["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(GitHubError::ApiError { status, message });
        }

        let json: Value = response.json().await?;
        json.get("tree")
            .and_then(|tree| tree.get("sha"))
            .and_then(|sha| sha.as_str())
            .map(String::from)
            .ok_or_else(|| {
                GitHubError::ParseError("Failed to extract tree SHA from response".to_string())
            })
    }

    /// ファイル内容のBLOBを作成する
    pub async fn create_blob(
        &self,
        owner: &str,
        repo: &str,
        content: &str,
    ) -> Result<String, GitHubError> {
        let path = format!("/repos/{}/{}/git/blobs", owner, repo);
        let body = serde_json::json!({
            "content": content,
            "encoding": "utf-8"
        });

        let response = self.post(&path, &body).await?;
        let status = response.status();

        if !status.is_success() {
            let error_json: Value = response.json().await?;
            let message = error_json["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(GitHubError::ApiError { status, message });
        }

        let json: Value = response.json().await?;
        json.get("sha")
            .and_then(|sha| sha.as_str())
            .map(String::from)
            .ok_or_else(|| {
                GitHubError::ParseError("Failed to extract blob SHA from response".to_string())
            })
    }

    /// BLOBを含むツリーを作成する
    pub async fn create_tree(
        &self,
        owner: &str,
        repo: &str,
        base_tree: &str,
        path: &str,
        blob_sha: &str,
    ) -> Result<String, GitHubError> {
        let api_path = format!("/repos/{}/{}/git/trees", owner, repo);
        let body = serde_json::json!({
            "base_tree": base_tree,
            "tree": [{
                "path": path,
                "mode": "100644",
                "type": "blob",
                "sha": blob_sha
            }]
        });

        let response = self.post(&api_path, &body).await?;
        let status = response.status();

        if !status.is_success() {
            let error_json: Value = response.json().await?;
            let message = error_json["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(GitHubError::ApiError { status, message });
        }

        let json: Value = response.json().await?;
        json.get("sha")
            .and_then(|sha| sha.as_str())
            .map(String::from)
            .ok_or_else(|| {
                GitHubError::ParseError("Failed to extract tree SHA from response".to_string())
            })
    }

    /// 新しいコミットを作成する
    pub async fn create_commit(
        &self,
        owner: &str,
        repo: &str,
        message: &str,
        tree_sha: &str,
        parent_sha: &str,
    ) -> Result<String, GitHubError> {
        let path = format!("/repos/{}/{}/git/commits", owner, repo);
        let body = serde_json::json!({
            "message": message,
            "tree": tree_sha,
            "parents": [parent_sha]
        });

        let response = self.post(&path, &body).await?;
        let status = response.status();

        if !status.is_success() {
            let error_json: Value = response.json().await?;
            let message = error_json["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(GitHubError::ApiError { status, message });
        }

        let json: Value = response.json().await?;
        json.get("sha")
            .and_then(|sha| sha.as_str())
            .map(String::from)
            .ok_or_else(|| {
                GitHubError::ParseError("Failed to extract commit SHA from response".to_string())
            })
    }

    /// ブランチの先端を更新する
    pub async fn update_branch_reference(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        commit_sha: &str,
    ) -> Result<(), GitHubError> {
        let path = format!("/repos/{}/{}/git/refs/heads/{}", owner, repo, branch);
        let body = serde_json::json!({
            "sha": commit_sha,
            "force": false
        });

        let response = self.patch(&path, &body).await?;
        let status = response.status();

        if !status.is_success() {
            let error_json: Value = response.json().await?;
            let message = error_json["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(GitHubError::ApiError { status, message });
        }

        Ok(())
    }

    /// プルリクエストを作成する
    ///
    /// # 引数
    /// * `owner` - リポジトリのオーナー名
    /// * `repo` - リポジトリ名
    /// * `base` - マージ先となるベースブランチ名
    /// * `head` - マージ元となるヘッドブランチ名
    /// * `title` - プルリクエストのタイトル
    /// * `body` - プルリクエストの説明文
    ///
    /// # 戻り値
    /// * `Ok(())` - プルリクエストの作成に成功
    /// * `Err(GitHubError)` - APIリクエストが失敗した場合のエラー
    pub async fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
        head: &str,
        title: &str,
        body: &str,
    ) -> Result<(), GitHubError> {
        let path = format!("/repos/{}/{}/pulls", owner, repo);
        let request_body = serde_json::json!({
            "title": title,
            "body": body,
            "base": base,
            "head": head
        });

        let response = self.post(&path, &request_body).await?;
        let status = response.status();

        if !status.is_success() {
            let error_json: Value = response.json().await?;
            let message = error_json["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(GitHubError::ApiError { status, message });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_client_creation() {
        let token = "test_token".to_string();
        let client = GitHubClient::new(token.clone());
        assert_eq!(client.token.as_str(), token);
        assert_eq!(client.base_url, "https://api.github.com");
    }

    #[tokio::test]
    async fn test_get_base_branch_sha() {
        use serde_json::json;

        let mock_response = json!({
            "ref": "refs/heads/main",
            "object": {
                "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
                "type": "commit",
                "url": "https://api.github.com/repos/octocat/Hello-World/git/commits/6dcb09b5b57875f334f61aebed695e2e4193db5e"
            }
        });

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/repos/owner/repo/git/ref/heads/main")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let mut client = GitHubClient::new("test_token".to_string());
        client.base_url = server.url();

        let result = client.get_base_branch_sha("owner", "repo", "main").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "6dcb09b5b57875f334f61aebed695e2e4193db5e");
    }

    #[tokio::test]
    async fn test_create_branch() {
        use serde_json::json;

        let expected_body = json!({
            "ref": "refs/heads/new-feature",
            "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e"
        });

        let mut server = mockito::Server::new_async().await;
        let _mock = server.mock("POST", "/repos/owner/repo/git/refs")
            .match_body(mockito::Matcher::Json(expected_body))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ref": "refs/heads/new-feature", "object": {"sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e"}}"#)
            .create_async()
            .await;

        let mut client = GitHubClient::new("test_token".to_string());
        client.base_url = server.url();

        let result = client
            .create_branch(
                "owner",
                "repo",
                "new-feature",
                "6dcb09b5b57875f334f61aebed695e2e4193db5e",
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_pull_request() {
        use serde_json::json;

        let expected_body = json!({
            "title": "テスト PR",
            "body": "PR の本文",
            "base": "main",
            "head": "feature-branch"
        });

        let mut server = mockito::Server::new_async().await;
        let _mock = server.mock("POST", "/repos/owner/repo/pulls")
            .match_body(mockito::Matcher::Json(expected_body))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"number": 1, "state": "open"}"#)
            .create_async()
            .await;

        let mut client = GitHubClient::new("test_token".to_string());
        client.base_url = server.url();

        let result = client
            .create_pull_request(
                "owner",
                "repo",
                "main",
                "feature-branch",
                "テスト PR",
                "PR の本文",
            )
            .await;

        assert!(result.is_ok());
    }
}
