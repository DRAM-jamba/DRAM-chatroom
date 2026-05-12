
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("Json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("DB error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Else: {0}")]
    Else(String),

    #[error("Not Found")]
    NotFound,

    #[error("Forbidden")]
    Forbidden(String),

    #[error("Invalid input")]
    InvalidInput(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),
}