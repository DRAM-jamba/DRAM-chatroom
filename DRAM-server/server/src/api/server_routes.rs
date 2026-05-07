use axum::{Json, Router, extract::{Path, State}, routing::get};
use serde_json::{Value, json};
use crate::{errors::api_error::ApiError, logic::{auth_logic::l_check_active_user, server_logic::{l_add_user_to_server, l_connect_user_to_server, l_delete_user_from_server, l_set_user_nickname}}, modules::server_state::ServerState};

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/add", get(r_add_server))
        .route("/connect/{user_key}", get(r_connect_to_server))
        .route("/leave", get(r_leave_server))
        .route("/forget/{user_key}", get(r_forget_server))
        .route("/set/nickname/{user_key}/{nickname}", get(r_set_nickname))
}

async fn r_add_server(State(server_state): State<ServerState>) -> Result<Json<Value>, ApiError> {
    match l_add_user_to_server(server_state.db_pool.clone()).await {
        Ok(user_key) => {
            Ok(Json(json!({
                "user_key": user_key
            })))
        }
        Err(e) => {
            Err(e)
        }
    }
}

async fn r_connect_to_server(State(server_state): State<ServerState>, Path(user_key): Path<String>) -> Result<Json<Value>, ApiError> {
    match l_connect_user_to_server(server_state.db_pool.clone(), user_key).await {
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

async fn r_forget_server(State(server_state): State<ServerState>, Path(user_key): Path<String>) -> Result<(), ApiError> {
    
    match l_check_active_user(server_state.active_users.clone(), &user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    match l_delete_user_from_server(server_state.db_pool.clone(), server_state.active_sessions.clone(), user_key).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

async fn r_set_nickname(State(server_state): State<ServerState>, 
                        Path((user_key, nickname)): Path<(String, String)>) -> Result<Json<Value>, ApiError> {
    match l_set_user_nickname(server_state.db_pool.clone(), user_key, nickname).await {
        Ok(()) => Ok(Json(json!({"response":"nickname was changed successfully"}))),
        Err(e) => Err(e)
    }
}