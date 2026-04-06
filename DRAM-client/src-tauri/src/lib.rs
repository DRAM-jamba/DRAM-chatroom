use crate::error::AppError;
use crate::events::{emit_joined_session, emit_session_update};
use crate::state::{AppState, ConnectionState, SessionState};
use crate::api::ServerApi;
use crate::websocket::WsClient;
use tauri::{AppHandle, State};
use tokio_tungstenite::tungstenite::client;
use serde::{Deserialize, Serialize};

mod error;
mod events;
mod state;
mod api;
pub mod websocket;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn add(
    ip: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    ip.parse::<std::net::Ipv4Addr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;

    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.add();

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to connect: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected connection: {}", e)))?;

    let user_key: String = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Network(format!("Failed to parse response: {}", e)))?
        .get("user_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Network("Missing user_key in response".into()))?
        .to_string();

    state.known_servers.lock().await.insert(ip.clone(), user_key);
    events::emit_connected(&app);
    Ok(())
}

#[tauri::command]
async fn connect(
    ip: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    ip.parse::<std::net::Ipv4Addr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;
    
    let user_key = state.known_servers.lock().await
        .get(&ip)
        .cloned()
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — use add first", ip)))?;
    
    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.connect(&user_key);
    let client = reqwest::Client::new();
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to connect: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected connection: {}", e)))?;

    *state.current_ip.lock().await = Some(ip.clone());
    *state.connection.lock().await = ConnectionState::JoinedServer { ip: ip.clone() };
    events::emit_connected(&app);
    Ok(())
}

#[tauri::command]
async fn disconnect(state: State<'_, AppState>) -> Result<(), AppError> {
    websocket::stop_heartbeat(&state).await;
    *state.connection.lock().await = ConnectionState::Disconnected;
    *state.session.lock().await = SessionState::Idle;
    Ok(())
}

#[tauri::command]
async fn create_session(
    name: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;

    let user_key = state.known_servers.lock().await
        .get(&ip)
        .cloned()
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add a server first", ip)))?;
    
    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.create_session(&user_key, &name);
    let response = state.client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to create session: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected creation: {}", e)))?;

    let session_key: String = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Network(format!("Failed to parse response: {}", e)))?
        .get("session_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Network("Missing session_key in response".into()))?
        .to_string();

    
    Ok(())
}

#[tauri::command]
async fn join_session(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;

    let user_key = state.known_servers.lock().await
        .get(&ip)
        .cloned()
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add a server first", ip)))?;
    
    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.join_session(&user_key, &session_id);
    let response = state.client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to join session: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected join: {}", e)))?;

    let response_value: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Network(format!("Failed to parse response: {}", e)))?;

    let payload: events::SessionPayload = if let Some(inner) = response_value.get("payload") {
        serde_json::from_value(inner.clone())
            .map_err(|e| AppError::Network(format!("Failed to parse session payload: {}", e)))?
    } else {
        serde_json::from_value(response_value)
            .map_err(|e| AppError::Network(format!("Failed to parse session payload: {}", e)))?
    };

    *state.session.lock().await = SessionState::JoinedSession {
        session_id: payload.session_id.clone(),
        participants: payload.participants.clone(),
    };

    //let ws_url = api.ws(&user_key);
    //let ws_client = websocket::WsClient::connect(&ws_url, app.clone())
    //    .await
    //    .map_err(|e| AppError::Network(format!("Failed to open websocket: {}", e)))?;

    //*state.connection.lock().await = ConnectionState::Connected(ws_client);
    //websocket::start_heartbeat(&state).await;

    emit_joined_session(&app);
    emit_session_update(&app, payload);
    Ok(())
}

#[tauri::command]
async fn send_message(body: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let conn = state.connection.lock().await;
    match &*conn {
        ConnectionState::Connected(client) => client.send(&body).await,
        _ => Err(AppError::Network("Not connected".into())),
    }
}

#[tauri::command]
async fn leave_session(state: State<'_, AppState>) -> Result<(), AppError> {
    *state.session.lock().await = SessionState::Idle;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_keyring::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            add,
            connect,
            disconnect,
            create_session,
            join_session,
            leave_session,
            send_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
