use std::sync::Arc;

use axum::{Json, Router, response::IntoResponse, routing::get};
use serde_json::{Value, json};
use tokio::{net::TcpListener, sync::Mutex};

use crate::{api::{server_routes, session_routes }, errors::api_error::ApiError, modules::{state::AppState, user::User}};

mod api;
mod modules;
mod logic;
mod data_logic;
mod errors;

#[tokio::main]
async fn main() {

    let initial_users = vec![
        User {id: 1, user_key: "holy shit".into(), nickname: "ho".into(), related_session_keys: [].to_vec(), last_time_seen: 123 },
        User {id: 2, user_key: "oh my god!".into(), nickname: "dddOh".into(), related_session_keys: [].to_vec(), last_time_seen: 3233 },
    ];
    
    let state = AppState {
        users: Arc::new(Mutex::new(initial_users)),
        secret: "antihype".into(),
    };

    let state_for_server = state.clone();

    
    let session_router = api::session_routes::router().with_state(state_for_server.clone());
    let server_router = api::server_routes::router().with_state(state_for_server.clone());
    
    let app = Router::new()
        .route("/", get(server_check))
        .route("/error", get(error_check))
        .nest("/session", session_router)
        .nest("/server", server_router);

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
    println!("Server started");
}

async fn server_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "message": "server is running",
    }))
}

async fn error_check() -> Result<Json<Value>, ApiError> {
    Err(ApiError::NotFound)
}