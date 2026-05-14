use axum::{Json, Router, extract::{State}, routing::{delete, patch, post, put}};
use serde_json::{Value, json};
use crate::{errors::api_error::ApiError, logic::{auth_logic::l_ensure_user_not_in_session, server_logic::{l_add_user_to_server, l_connect_user_to_server, l_delete_user_from_server, l_set_user_nickname}}, modules::{request_bodies::{UserKey, UserKeyNickname}, server_state::ServerState}};

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/add", post(r_add_server))
        .route("/connect", put(r_connect_to_server))
        .route("/leave", delete(r_leave_server))
        .route("/forget", delete(r_forget_server))
        .route("/nickname", patch(r_set_nickname))
}

async fn r_add_server(State(server_state): State<ServerState>) -> Result<Json<Value>, ApiError> {
    match l_add_user_to_server(server_state.db_pool.clone()).await {
        Ok(user_key) => {
            Ok(Json(json!({
                "user_key": user_key
            })))
        }
        Err(e) => Err(e)
    }
}

async fn r_connect_to_server(State(server_state): State<ServerState>, 
                             Json(payload): Json<UserKey>) -> Result<Json<Value>, ApiError> {
    match l_connect_user_to_server(server_state.db_pool.clone(), payload.user_key).await {
        Ok(auth_token) => {
            Ok(Json(json!({
                "auth_token": auth_token
            })))
        }
        Err(e) => Err(e)
    }
}

// TODO: finish
async fn r_leave_server(State(_server_state): State<ServerState>) -> Result<Json<Value>, ApiError> {
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

async fn r_forget_server(State(server_state): State<ServerState>, 
                         Json(payload): Json<UserKey>) -> Result<Json<Value>, ApiError> {
    
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &payload.user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    };
    
    match l_delete_user_from_server(server_state.db_pool.clone(), server_state.active_sessions.clone(), payload.user_key).await {
        Ok(()) => Ok(Json(json!({"respone":"user was deleted successfully"}))),
        Err(e) => Err(e)
    }
}

async fn r_set_nickname(State(server_state): State<ServerState>, 
                        Json(payload): Json<UserKeyNickname>) -> Result<Json<Value>, ApiError> {
    match l_ensure_user_not_in_session(server_state.active_users.clone(), &payload.user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    };
    
    match l_set_user_nickname(server_state.db_pool.clone(), payload.user_key, payload.nickname).await {
        Ok(()) => Ok(Json(json!({"response":"nickname was changed successfully"}))),
        Err(e) => Err(e)
    }
}