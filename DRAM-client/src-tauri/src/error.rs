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
}

// Convert AppError to String for Tauri's InvokeError
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        tauri::ipc::InvokeError::from_anyhow(e.into())
    }
}