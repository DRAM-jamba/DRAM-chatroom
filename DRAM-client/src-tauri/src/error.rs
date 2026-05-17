use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Convert AppError to String for Tauri's InvokeError
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

// Convert reqwest::Error to AppError
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_status() {
            AppError::Protocol(format!("Server rejected request: {}", e))
        } else {
            AppError::Network(format!("Connection failed: {}", e))
        }
    }
}
