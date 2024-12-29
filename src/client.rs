use reqwest::{Client, Response};
use crate::auth::{AuthToken, build_auth_headers};

pub struct GitHubClient {
    http: Client,
    token: AuthToken,
    base_url: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        Self {
            http: Client::new(),
            token: AuthToken::new(token),
            base_url: "https://api.github.com".to_string(),
        }
    }

    pub async fn get(&self, path: &str) -> reqwest::Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let headers = build_auth_headers(self.token.as_str());
        self.http.get(url).headers(headers).send().await
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
