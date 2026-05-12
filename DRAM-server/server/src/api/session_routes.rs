use axum::{Json, Router, extract::{Path, State, WebSocketUpgrade}, response::IntoResponse, routing::get};
use serde_json::{Value, json};
use crate::{errors::api_error::ApiError, logic::{auth_logic::{l_ensure_user_not_in_session}, chat_logic::l_connection_handler, session_logic::{l_add_session, l_create_session, l_delete_session_by_owner, l_forget_session, l_get_session_list}}, modules::server_state::ServerState};

pub fn router() -> Router<ServerState> {
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

async fn r_get_session_list(State(server_state): State<ServerState>,
                          Path(user_key): Path<String>) -> Result<Json<Value>, ApiError> {
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    match l_get_session_list(server_state.db_pool.clone(), &user_key).await {
        Ok(session_list) => {
            Ok(Json(json!({
                "user_sessions": session_list
            })))
        },
        Err(e) => Err(e)
    }
}

async fn r_create_session(State(server_state): State<ServerState>,
                        Path((user_key, session_name)): Path<(String, String)>) -> Result<Json<Value>, ApiError> {
    
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }

    match l_create_session(server_state.db_pool.clone(), &user_key, &session_name).await {
        Ok(session_key) => Ok(Json(json!({
            "session_key": session_key
        }))),
        Err(e) => Err(e)
    }
}

async fn r_add_session(State(server_state): State<ServerState>,
                     Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    match l_add_session(server_state.db_pool.clone(), &user_key, &session_key).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn r_connect_to_session(State(server_state): State<ServerState>, 
                            Path((user_key, session_key)): Path<(String, String)>, 
                            ws: WebSocketUpgrade) -> Result<impl IntoResponse, ApiError> {
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
                       Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    match l_forget_session(server_state.db_pool.clone(), &user_key, &session_key).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn r_delete_session(State(server_state): State<ServerState>,
                        Path((user_key, session_key)): Path<(String, String)>) -> Result<(), ApiError> {
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    match l_delete_session_by_owner(server_state.db_pool.clone(), server_state.active_sessions.clone(), &user_key, &session_key).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}