use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    NotFound, // 404
    InvalidInput(String), // 400
    InternalError // 500
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::NotFound =>( 
                StatusCode::NOT_FOUND, "Data not found".to_string(),
            ),
            ApiError::InvalidInput(msg) => (
                StatusCode::BAD_REQUEST, msg
            ),
            ApiError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string()
            ),
        };
        
        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}