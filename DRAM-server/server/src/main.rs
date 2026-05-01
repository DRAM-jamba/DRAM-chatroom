use std::{env, process::exit, sync::Arc};

use axum::{Json, Router, http::HeaderValue, response::IntoResponse, routing::get};
use serde_json::{Value, json};
use tokio::{net::TcpListener};
use tower_http::cors::{Any, CorsLayer};
use sqlx::{postgres::PgPoolOptions};

use crate::{api::{server_routes, session_routes }, errors::api_error::ApiError, modules::{server_state::{ServerInfo, ServerState}}};

mod api;
mod modules;
mod logic;
mod data_logic;
mod errors;

#[tokio::main]
async fn main() {
    let db_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    };
    let pool = match PgPoolOptions::new().connect(&db_url).await {
        Ok(db_pool) => db_pool,
        Err(e) => {
            eprintln!("{e}");
            exit(2)
        }
    };
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{e}");
            exit(3)
        }
    };

    let server_state: ServerState = Arc::new(ServerInfo::new(pool));

    let session_router = session_routes::router();
    let server_router = server_routes::router();
    
    let cors_layer = CorsLayer::new().allow_methods(Any).allow_origin("http://127.0.0.1:8080".parse::<HeaderValue>().unwrap());

    let app = Router::new()
        .route("/", get(server_check))
        .route("/error", get(error_check))
        .nest("/session", session_router)
        .nest("/server", server_router)
        .layer(cors_layer)
        .with_state(server_state);

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