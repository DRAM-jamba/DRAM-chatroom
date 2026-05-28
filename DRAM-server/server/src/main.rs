use std::{env, process::exit, sync::Arc};

use axum::{Json, Router, response::IntoResponse, routing::get};
use chrono::Utc;
use dotenvy::dotenv;
use serde_json::{Value, json};
use tokio::{net::TcpListener};
use sqlx::{postgres::PgPoolOptions};

use crate::{api::{server_routes, session_routes }, errors::api_error::ApiError, modules::{active_nonce::NonceMap, server_state::{ServerInfo, ServerState}}};

mod api;
mod modules;
mod logic;
mod data_logic;
mod errors;
mod middleware;

#[tokio::main]
async fn main() {

    dotenv().ok();

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

    start_nonce_cleanup(server_state.active_nonce.clone());

    let session_router = session_routes::router(server_state.clone());
    let server_router = server_routes::router(server_state.clone());
    
    // let cors_layer = CorsLayer::new().allow_methods(Any).allow_origin("http://127.0.0.1:8080".parse::<HeaderValue>().unwrap());

    let app = Router::new()
        .route("/", get(server_check))
        .route("/error", get(error_check))
        .nest("/session", session_router)
        .nest("/server", server_router)
        // .layer(cors_layer)
        .with_state(server_state);

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Server started");
    axum::serve(listener, app).await.unwrap();
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

pub fn start_nonce_cleanup(active_nonce: NonceMap) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(60)
        );

        loop {
            interval.tick().await;

            let now = Utc::now();
            let before = active_nonce.len();

            // retain only nonces that haven't expired
            active_nonce.retain(|_, pending| pending.expires_at > now);

            let _removed = before - active_nonce.len();
            // if removed > 0 {
            //     tracing::info!("cleaned up {} expired nonces", removed);
            // }
        }
    });
}