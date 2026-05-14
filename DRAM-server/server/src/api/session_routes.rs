use axum::{Json, Router, extract::{State, WebSocketUpgrade}, response::IntoResponse, routing::{delete, get, post}};
use hyper::HeaderMap;
use serde_json::{Value, json};
use crate::{errors::api_error::ApiError, logic::{auth_logic::l_ensure_user_not_in_session, chat_logic::l_connection_handler, session_logic::{l_add_session, l_create_session, l_delete_session_by_owner, l_forget_session, l_get_session_list}}, modules::{request_bodies::{UserKey, UserKeySessionName, UserSessionKeys}, server_state::ServerState}};

pub fn router() -> Router<ServerState> {
    Router::new() // move user_key to body of request
        .route("/list", get(r_get_session_list))
        .route("/create", post(r_create_session))
        .route("/add", post(r_add_session))
        .route("/connect", get(r_connect_to_session))
        .route("/leave", delete(r_leave_session))
        .route("/forget", delete(r_forget_session))
        .route("/delete", delete(r_delete_session))
}

async fn r_get_session_list(State(server_state): State<ServerState>,
                            Json(payload): Json<UserKey>) -> Result<Json<Value>, ApiError> {
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &payload.user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    match l_get_session_list(server_state.db_pool.clone(), &payload.user_key).await {
        Ok(session_list) => {
            Ok(Json(json!({
                "user_sessions": session_list
            })))
        },
        Err(e) => Err(e)
    }
}

async fn r_create_session(State(server_state): State<ServerState>,
                          Json(payload): Json<UserKeySessionName>) -> Result<Json<Value>, ApiError> {
    
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &payload.user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }

    match l_create_session(server_state.db_pool.clone(), &payload.user_key, &payload.session_name).await {
        Ok(session_key) => Ok(Json(json!({
            "session_key": session_key
        }))),
        Err(e) => Err(e)
    }
}

async fn r_add_session(State(server_state): State<ServerState>,
                       Json(payload): Json<UserSessionKeys>) -> Result<Json<Value>, ApiError> {
    
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &payload.user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    match l_add_session(server_state.db_pool.clone(), &payload.user_key, &payload.session_key).await {
        Ok(()) => Ok(Json(json!({
            "respose": "session was added successfully"
        }))),
        Err(e) => Err(e)
    }
}

async fn r_connect_to_session(State(server_state): State<ServerState>, 
                              ws: WebSocketUpgrade, 
                              headers: HeaderMap) -> Result<impl IntoResponse, ApiError> {

    let user_key = headers.get("user_key")
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("")
                                .to_string();
    let session_key = headers.get("session_key")
                                   .and_then(|v| v.to_str().ok())
                                   .unwrap_or("")
                                   .to_string();

    match l_ensure_user_not_in_session(server_state.active_users.clone(), &user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }               
    l_connection_handler(server_state.clone(), user_key, session_key, ws).await
}

// TODO: we don't need it. finish websocketing is just make close request to websocket
async fn r_leave_session(State(_server_state): State<ServerState>) -> Result<Json<Value>, ApiError> {
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

async fn r_forget_session(State(server_state): State<ServerState>,
                          Json(payload): Json<UserSessionKeys>) -> Result<Json<Value>, ApiError> {
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &payload.user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    match l_forget_session(server_state.db_pool.clone(), &payload.user_key, &payload.session_key).await {
        Ok(()) => Ok(Json(json!({
            "respose": "session was forgotten successfully"
        }))),
        Err(e) => Err(e)
    }
}

async fn r_delete_session(State(server_state): State<ServerState>,
                          Json(payload): Json<UserSessionKeys>) -> Result<Json<Value>, ApiError> {
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &payload.user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    match l_delete_session_by_owner(server_state.db_pool.clone(), server_state.active_sessions.clone(), 
                                    &payload.user_key, &payload.session_key).await {
        Ok(()) => Ok(Json(json!({
            "respose": "session was deleted successfully"
        }))),
        Err(e) => Err(e)
    }
}