use crate::error::AppError;
use crate::state::{AppState, ConnectionState, SessionState};
use crate::api::ServerApi;
use crate::websocket::WsClient;
use tauri::{AppHandle, State};
use tokio_tungstenite::tungstenite::client;

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
    // Validate IP address format
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
    // Validate IP address format
    ip.parse::<std::net::Ipv4Addr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;
    
    // Send connect request
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

    join_session(session_key, state, app);
    Ok(())
}

#[tauri::command]
async fn join_session(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
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
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            connect,
            disconnect,
            join_session,
            leave_session,
            send_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
