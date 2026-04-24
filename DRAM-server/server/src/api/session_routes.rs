use axum::{Json, Router, extract::{Path, State, WebSocketUpgrade}, http::StatusCode, response::IntoResponse, routing::get};
use serde::Serialize;
use serde_json::{Value, json};
use crate::{errors::api_error::ApiError, logic::{chat_logic::l_connection_handler, session_logic::{l_add_session_by_session_key, l_create_session, l_delete_session_by_owner, l_forget_session_by_user, l_get_user_related_session_list}}, modules::session_chat::SessionMap};

pub fn router() -> Router<SessionMap> {
    Router::new() // move user_key to body of request
        .route("/list/{user_key}", get(r_get_session_list))
        .route("/create/{user_key}/{session_name}", get(r_create_session))
        .route("/add/{user_key}/{session_key}", get(r_add_session))
        .route("/connect/{user_key}/{session_key}", get(r_connect_to_session))
        .route("/leave", get(r_leave_session))
        .route("/forget/{user_key}/{session_key}", get(r_forget_session))
        // user_key to proof that session is created by user
        .route("/delete/{user_key}/{session_key}", get(r_delete_session))
}

async fn r_get_session_list(State(_): State<SessionMap>,
                          Path(user_key): Path<String>) -> Result<Json<Value>, ApiError> {
    match l_get_user_related_session_list(user_key) {
        Ok(v) => {
            Ok(Json(json!({
                "related_sessions": v
            })))
        },
        Err(e) => Err(e)
    }
}

async fn r_create_session(State(_): State<SessionMap>,
                        Path((user_key, session_name)): Path<(String, String)>) -> Result<(), ApiError> {
    
    match l_create_session(user_key, session_name) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn r_add_session(State(_): State<SessionMap>,
                     Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match l_add_session_by_session_key(user_key, session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn r_connect_to_session(State(active_sessions): State<SessionMap>, 
                            Path((user_key, session_key)): Path<(String, String)>, 
                            ws: WebSocketUpgrade) -> Result<impl IntoResponse, ApiError> {
    l_connection_handler(active_sessions, user_key, session_key, ws).await
}

// TODO: we don't need it. finish websocketing is just make close request to websocket
async fn r_leave_session(State(_): State<SessionMap>) -> Result<Json<Value>, ApiError> {
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

async fn r_forget_session(State(_): State<SessionMap>,
                       Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match l_forget_session_by_user(user_key, session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn r_delete_session(State(_): State<SessionMap>,
                        Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match l_delete_session_by_owner(&user_key, &session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}