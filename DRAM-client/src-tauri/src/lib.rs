use crate::error::AppError;
use crate::events::{emit_joined_session, emit_session_update};
use crate::state::{AppState, ConnectionState, SessionState};
use crate::api::ServerApi;
use crate::websocket::WsClient;
use tauri::{AppHandle, State};
use tokio_tungstenite::tungstenite::client;
use serde::{Deserialize, Serialize};
use tauri::Manager;

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
    nickname: String,
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

    state.add_server(ip.clone(), nickname, user_key).await
        .map_err(|e| AppError::Network(format!("Failed to save server: {}", e)))?;
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
    
    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — use add first", ip)))?;
    
    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.connect(&server.user_key);
    let client = reqwest::Client::new();
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to connect: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected connection: {}", e)))?;

    *state.current_ip.lock().await = Some(ip.clone());
    *state.connection.lock().await = ConnectionState::JoinedServer;
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

    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add a server first", ip)))?;
    
    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.create_session(&server.user_key, &name);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to create session: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected session creation: {}", e)))?;

    let _session_key: String = response
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

    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add a server first", ip)))?;
    
    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.join_session(&server.user_key, &session_id);
    let client = reqwest::Client::new();
    let response = client
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

    *state.session.lock().await = SessionState::JoinedSession;

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
        ConnectionState::Connected => {
            // TODO: Send via WebSocket client
            Ok(())
        },
        _ => Err(AppError::Network("Not connected".into())),
    }
}

#[tauri::command]
async fn set_nickname(
    new_nickname: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    println!("Attempting to set nickname to '{}' for server at {}", new_nickname, state.current_ip.lock().await.as_ref().unwrap_or(&"None".to_string()));
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;
    println!("Current state IP: {}", state.current_ip.lock().await.as_ref().unwrap());
    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add a server first", ip)))?;

    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.set_nickname(&server.user_key, &new_nickname);
    let client = reqwest::Client::new();
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to set nickname: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected nickname change: {}", e)))?;
    
    Ok(())
}

#[tauri::command]
async fn leave_session(state: State<'_, AppState>) -> Result<(), AppError> {
    *state.session.lock().await = SessionState::Idle;
    Ok(())
}

#[tauri::command]
async fn get_servers(state: State<'_, AppState>) -> Result<Vec<state::PersistedServer>, AppError> {
    Ok(state.servers.lock().await.clone())
}

#[tauri::command]
async fn remove_server(
    id: String, 
    state: State<'_, AppState>
) -> Result<(), AppError> {
    println!("Attempting to forget server with ID '{}'", &id);
    let server = state.get_server_by_id(&id)
        .await
        .ok_or_else(|| AppError::Auth(format!("Server does not exist: {}", &id)))?;

    let api = ServerApi::new(&format!("http://{}", server.ip));
    let url = api.forget(&server.user_key);
    let client = reqwest::Client::new();
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to forget server: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected forget: {}", e)))?;

    state.remove_server(&server.ip).await
        .map_err(|e| AppError::Network(format!("Failed to remove server: {}", e)))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_keyring::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_state = AppState::new(app_handle);
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            add,
            connect,
            disconnect,
            create_session,
            join_session,
            leave_session,
            set_nickname,
            send_message,
            get_servers,
            remove_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
