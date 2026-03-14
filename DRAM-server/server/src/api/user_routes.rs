use axum::{Json, Router, extract::{Path, State}, http::StatusCode, routing::get};
use serde::Serialize;
use crate::modules::state::AppState;

#[derive(Clone, Serialize)]
pub struct User {
    pub id: u64,
    pub name: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/{id}", get(get_users))
}

async fn list_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let guard = state.users.lock().await;
    Json(guard.clone())
}

async fn get_users(State(state): State<AppState>, Path(id): Path<u64>) -> Result<Json<User>, StatusCode> {
    let guard = state.users.lock().await;
    if let Some(user ) = guard.iter().find(|u| u.id == id).cloned() {
        Ok(Json(user))
    }
    else {
        Err(StatusCode::NOT_FOUND)
    }
}