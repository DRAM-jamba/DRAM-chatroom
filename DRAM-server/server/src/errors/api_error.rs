use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use crate::{errors::app_error::AppError, };

#[derive(Debug)]
pub enum ApiError {
    NotFound, // 404
    InvalidInput(String), // 400
    InternalError, // 500
    Unauthorized, // 401
    Forbidden(String), // 403 
    Conflict(String), // 409
}

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::NotFound => ApiError::NotFound,
            AppError::Database(sqlx::Error::RowNotFound) => ApiError::NotFound,
            AppError::Database(sqlx::Error::Database(e)) if e.constraint().is_some() => ApiError::Conflict(e.to_string()),
            AppError::Database(e) => ApiError::InvalidInput(e.to_string()),
            AppError::Forbidden(e) => ApiError::Forbidden(e),
            AppError::InvalidInput(e) => ApiError::InvalidInput(e),
            _ => ApiError::InternalError
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::NotFound =>( 
                StatusCode::NOT_FOUND, 
                "Data not found".to_string(),
            ),
            ApiError::InvalidInput(msg) => (
                StatusCode::BAD_REQUEST, 
                msg
            ),
            ApiError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string()
            ),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "User is unauthorized".to_string()
            ),
            ApiError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                msg
            ),
            ApiError::Conflict(msg) => (
                StatusCode::CONFLICT,
                msg
            ),
        };
        
        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}