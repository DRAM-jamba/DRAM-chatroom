use axum::{extract::{Request, State}, middleware::Next, response::Response};
use hyper::{StatusCode, header::AUTHORIZATION};

use crate::{errors::api_error::ApiError, logic::auth_logic::l_is_valid, modules::server_state::ServerState};

pub async fn auth_middle(
    State(_server_state): State<ServerState>, // for future
    request: Request,
    next: Next
) -> Result<Response, StatusCode> {

    let token = extract_token(&request)?;

    let _claims = l_is_valid(&token).await
        .map_err(|e| match e {
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(next.run(request).await)
}


fn extract_token(request: &Request) -> Result<String, StatusCode> {
    let header = request
        .headers()
        .get(AUTHORIZATION)
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(token.to_string())
}