use axum::{Json, Router, extract::{Path, State, WebSocketUpgrade}, http::StatusCode, response::IntoResponse, routing::get};
use serde::Serialize;
use serde_json::{Value, json};
use crate::{errors::api_error::ApiError, logic::{chat_logic::connection_handler, session_logic::{add_session_by_session_key, create_session_l, delete_session_by_owner, forget_session_by_user, get_user_related_session_list}}, modules::session_chat::SessionMap};

pub fn router() -> Router<SessionMap> {
    Router::new() // move user_key to body of request
        .route("/list/{user_key}", get(get_session_list))
        .route("/create/{user_key}/{session_name}", get(create_session))
        .route("/add/{user_key}/{session_key}", get(add_session))
        .route("/connect/{user_key}/{session_key}", get(connect_to_session))
        .route("/leave", get(leave_session))
        .route("/forget/{user_key}/{session_key}", get(forget_session))
        // user_key to proof that session is created by user
        .route("/delete/{user_key}/{session_key}", get(delete_session))
}

async fn get_session_list(State(_): State<SessionMap>,
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

async fn create_session(State(_): State<SessionMap>,
                        Path((user_key, session_name)): Path<(String, String)>) -> Result<(), ApiError> {
    
    match create_session_l(user_key, session_name) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn add_session(State(_): State<SessionMap>,
                     Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match add_session_by_session_key(user_key, session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn connect_to_session(State(active_sessions): State<SessionMap>, 
                            Path((user_key, session_key)): Path<(String, String)>, 
                            ws: WebSocketUpgrade) -> Result<impl IntoResponse, ApiError> {
    connection_handler(active_sessions, user_key, session_key, ws).await
}

// TODO: we don't need it. finish websocketing is just make close request to websocket
async fn leave_session(State(_): State<SessionMap>) -> Result<Json<Value>, ApiError> {
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

async fn forget_session(State(_): State<SessionMap>,
                       Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match forget_session_by_user(user_key, session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn delete_session(State(_): State<SessionMap>,
                        Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match delete_session_by_owner(&user_key, &session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}