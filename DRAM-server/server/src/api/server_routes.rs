use axum::{Json, Router, extract::State, middleware, routing::{delete, patch, post, put, get}};
use serde_json::{Value, json};
use crate::{errors::api_error::ApiError, logic::{auth_logic::{l_create_challenge, l_ensure_user_not_in_session, l_handle_challenge, l_refresh_token}, server_logic::{l_add_user_to_server, l_connect_user_to_server, l_delete_user_from_server, l_set_user_nickname}}, middleware::auth_middleware::auth_middle, modules::{request_bodies::{ChallengeResponse, PublicKey, UserKey, UserKeyNickname}, server_state::ServerState}};

pub fn router(server_state: ServerState) -> Router<ServerState> {
    Router::new()
        .route("/connect", put(r_connect_to_server))
        .route("/leave", delete(r_leave_server))
        .route("/nickname", patch(r_set_nickname))
        .route("/refresh_token", patch(r_refresh_token))
        .route_layer(middleware::from_fn_with_state(
            server_state.clone(), 
            auth_middle
        ))
        .route("/add", post(r_add_server))
        .route("/forget", delete(r_forget_server))
        .route("/challenge", get(r_create_challenge))
        .route("/token", post(r_handle_challenge))
}

async fn r_add_server(State(server_state): State<ServerState>,
                      Json(payload): Json<PublicKey>) -> Result<Json<Value>, ApiError> {
    match l_add_user_to_server(server_state.db_pool.clone(), payload.public_key).await {
        Ok(user_key) => {
            Ok(Json(json!({
                "user_key": user_key
            })))
        }
        Err(e) => Err(e)
    }
}

async fn r_create_challenge(State(server_state): State<ServerState>,
                      Json(payload): Json<UserKey>) -> Result<Json<Value>, ApiError> {
    match l_create_challenge(server_state, &payload.user_key).await {
        Ok(challenge) => {
            Ok(Json(json!({
                "challenge": challenge
            })))
        }
        Err(e) => Err(e)
    }
}

async fn r_handle_challenge(State(server_state): State<ServerState>,
                      Json(payload): Json<ChallengeResponse>) -> Result<Json<Value>, ApiError> {
    match l_handle_challenge(server_state, payload).await {
        Ok(token) => {
            Ok(Json(json!({
                "token": token
            })))
        }
        Err(e) => Err(e)
    }
}

async fn r_refresh_token(State(server_state): State<ServerState>,
                      Json(payload): Json<UserKey>) -> Result<Json<Value>, ApiError> {
    match l_refresh_token(server_state.db_pool.clone(), &payload.user_key).await {
        Ok(token) => {
            Ok(Json(json!({
                "token": token
            })))
        }
        Err(e) => Err(e)
    }
}

async fn r_connect_to_server(State(server_state): State<ServerState>, 
                             Json(payload): Json<UserKey>) -> Result<Json<Value>, ApiError> {
    match l_connect_user_to_server(server_state.db_pool.clone(), payload.user_key).await {
        Ok(()) => {
            Ok(Json(json!({
                "message": "congratulations! you successfully connected to server!"
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