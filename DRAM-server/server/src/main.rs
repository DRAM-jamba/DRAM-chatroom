use std::sync::Arc;

use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use serde_json::json;
use tokio::{net::TcpListener, sync::Mutex};

use crate::{api::user_routes::User, modules::state::AppState};

mod api;
mod modules;



#[tokio::main]
async fn main() {

    let initial_users = vec![
        User {id: 1, name: "holy shit".into() },
        User {id: 2, name: "oh my god!".into() },
    ];
    
    let state = AppState {
        users: Arc::new(Mutex::new(initial_users)),
        secret: "antihype".into(),
    };

    let state_for_server = state.clone();

    
    let users_router = api::user_routes::router().with_state(state_for_server.clone());
    
    let app = Router::new()
        .route("/", get(server_check))
        .nest("/api", users_router);

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