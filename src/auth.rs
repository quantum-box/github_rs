use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};

#[derive(Clone)]
pub struct AuthToken(pub String);

impl AuthToken {
    pub fn new<S: Into<String>>(token: S) -> Self {
        Self(token.into())
    }

    pub fn from_env() -> Result<Self, std::env::VarError> {
        use tracing::{debug, info};

        dotenvy::dotenv().ok();
        info!(target: "auth", "Loading GitHub token from environment");
        let token = std::env::var("GITHUB_TOKEN")?;
        debug!(target: "auth", token_prefix = %&token[..10], "Token loaded successfully");
        Ok(Self(token))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn build_auth_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let auth_value = format!("token {}", token); // GitHub API expects "token" prefix
    println!("Debug - Auth header prefix: token"); // Debug log without exposing full token
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value).expect("Invalid token format"),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_str("application/vnd.github.v3+json").unwrap(),
    );
    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token_creation() {
        let token = "test_token".to_string();
        let auth = AuthToken::new(token.clone());
        assert_eq!(auth.as_str(), token);
    }

    #[test]
    fn test_build_auth_headers() {
        let token = "test_token";
        let headers = build_auth_headers(token);
        assert!(headers.contains_key(AUTHORIZATION));
        assert!(headers.contains_key(ACCEPT));

        if let Some(auth_value) = headers.get(AUTHORIZATION) {
            assert_eq!(auth_value.to_str().unwrap(), format!("token {}", token));
        }
    }
}
