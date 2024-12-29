use reqwest::{Client, Response};
use crate::auth::{AuthToken, build_auth_headers};

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
            reqwest::header::HeaderValue::from_static("github-rs-client")
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
        self.http
            .post(url)
            .headers(headers)
            .json(body)
            .send()
            .await
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
}
