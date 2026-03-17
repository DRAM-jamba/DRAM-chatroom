use axum::{Json, Router, extract::{Path, State}, http::StatusCode, routing::get};
use serde::Serialize;
use serde_json::{Value, json};
use crate::{logic::server_logic::{add_user_to_server, connect_user_to_server}, modules::{api_error::ApiError, state::AppState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/add", get(add_server))
        .route("/connect/{user_key}", get(connect_to_server))
        .route("/leave", get(leave_server))
        .route("/forget/{user_key}", get(forget_server))
        .route("/set/nickname/{nickname}", get(set_nickname))
}

// TODO: finish
async fn add_server(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    match add_user_to_server() {
        Ok(response) => {
            let (auth_token, user_key) = response;
            Ok(Json(json!({
                "auth_token": auth_token,
                "user_key": user_key
            })))
        }
        Err(e) => {
            Err(e)
        }
    }
}

// TODO: finish
async fn connect_to_server(State(state): State<AppState>, 
                           Path(user_key): Path<String>) -> Result<Json<Value>, ApiError> {
    
    let response = connect_user_to_server(user_key);
    match response {
        Ok(auth_token) => {
            Ok(Json(json!({
                "auth_token": auth_token
            })))
        }
        Err(e) => {
            Err(e)
        }
    }
}


// TODO: finish
async fn leave_server(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    if let Some("32") = Some("32") {
        Ok(Json(json!({
            "status": "leave_server in maintance",
            "message": "not done yet",
        })))
    }
    else {
        Err(ApiError::NotFound)
    }
}

// TODO: finish
async fn forget_server(State(state): State<AppState>, 
                       Path(user_key): Path<String>) -> Result<Json<Value>, ApiError> {
    if let Some("32") = Some("32") {
        Ok(Json(json!({
            "status": "leave_server in maintance",
            "message": "not done yet",
        })))
    }
    else {
        Err(ApiError::NotFound)
    }
}

// TODO: finish
async fn set_nickname(State(state): State<AppState>, 
                      Path(nickname): Path<String>) -> Result<Json<Value>, ApiError> {
    if let Some("32") = Some("32") {
        Ok(Json(json!({
            "status": "set_nickname in maintance",
            "message": "not done yet",
        })))
    }
    else {
        Err(ApiError::NotFound)
    }
}