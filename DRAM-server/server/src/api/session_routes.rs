use axum::{Json, Router, extract::{Path, State}, http::StatusCode, routing::get};
use serde::Serialize;
use serde_json::{Value, json};
use crate::{errors::api_error::ApiError, logic::session_logic::{add_session_by_session_key, create_session_l, delete_session_by_owner, forget_session_by_user, get_user_related_session_list}, modules::state::AppState};

pub fn router() -> Router<AppState> {
    Router::new() // move user_key to body of request
        .route("/list/{user_key}", get(get_session_list))
        .route("/create/{user_key}/{session_name}", get(create_session))
        .route("/add/{user_key}/{session_key}", get(add_session))
        .route("/connect/{session_key}", get(connect_to_session))
        .route("/leave", get(leave_session))
        .route("/forget/{user_key}/{session_key}", get(forget_session))
        // user_key to proof that session is created by user
        .route("/delete/{user_key}/{session_key}", get(delete_session))
}

async fn get_session_list(State(state): State<AppState>,
                          Path(user_key): Path<String>) -> Result<Json<Value>, ApiError> {
    match get_user_related_session_list(user_key) {
        Ok(v) => {
            Ok(Json(json!({
                "related_sessions": v
            })))
        },
        Err(e) => Err(e)
    }
}

async fn create_session(State(state): State<AppState>,
                        Path((user_key, session_name)): Path<(String, String)>) -> Result<(), ApiError> {
    
    match create_session_l(user_key, session_name) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn add_session(State(state): State<AppState>,
                     Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match add_session_by_session_key(user_key, session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

// TODO: finish
async fn connect_to_session(State(state): State<AppState>, 
                            Path(session_key): Path<String>) -> Result<Json<Value>, ApiError> {
    if let Some("32") = Some("32") {
        Ok(Json(json!({
            "status": "connect_to_session in maintance",
            "message": "not done yet",
        })))
    }
    else {
        Err(ApiError::NotFound)
    }
}

// TODO: finish
async fn leave_session(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    if let Some("32") = Some("32") {
        Ok(Json(json!({
            "status": "leave_session in maintance",
            "message": "not done yet",
        })))
    }
    else {
        Err(ApiError::NotFound)
    }
}

async fn forget_session(State(state): State<AppState>,
                       Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match forget_session_by_user(user_key, session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn delete_session(State(state): State<AppState>,
                        Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match delete_session_by_owner(&user_key, &session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}