use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded")]
    RateLimitError,

    #[error("Resource not found: {0}")]
    NotFoundError(String),

    #[error("Invalid request: {0}")]
    InvalidRequestError(String),

    #[error("GitHub API error: {status_code} - {message}")]
    ApiError { status_code: u16, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = GitHubError::AuthError("Invalid token".to_string());
        assert_eq!(error.to_string(), "Authentication error: Invalid token");

        let error = GitHubError::ApiError {
            status_code: 422,
            message: "Validation failed".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "GitHub API error: 422 - Validation failed"
        );
    }
}
