use crate::error::AppError;
use crate::state::{AppState, ConnectionState, SessionState};
use crate::api::ServerApi;
use tauri::{AppHandle, State, http, ipc};
use tauri::Manager;
use tokio_tungstenite::tungstenite::handshake::server;

mod error;
mod events;
mod state;
mod api;
pub mod websocket;

// Helper functions
async fn get_server_context(
    state: &State<'_, AppState>
) -> Result<(String, state::PersistedServer), AppError> {
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Session("Not connected to server".into()))?;

    let server = state.get_server(&ip).await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add a server first", ip)))?;
    
    Ok((ip, server))
}

async fn http_get(
    url: &str
) -> Result<reqwest::Response, AppError> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.error_for_status()?;
    Ok(response)
}

// Server commands
#[tauri::command]
async fn add_server(
    ip: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    ip.parse::<std::net::SocketAddr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;

    let api = ServerApi::new(&format!("http://{}", ip));
    let response = http_get(&api.add_server()).await?;
    let json_body: serde_json::Value = response.json().await?;
    let user_key = json_body
        .get("user_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Protocol("Server response missing 'user_key'".into()))?
        .to_string();
    let temp_nick = "".to_string();

    *state.current_ip.lock().await = Some(ip.clone());
    *state.connection.lock().await = ConnectionState::JoinedServer;
    
    state.add_server(ip, temp_nick, user_key).await
        .map_err(|e| AppError::Internal(format!("Failed to save: {}", e)))?;
    
    Ok(())
}

#[tauri::command]
async fn connect_server(
    ip: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    ip.parse::<std::net::SocketAddr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;

    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add it first", ip)))?;
    let api = ServerApi::new(&format!("http://{}", ip));
    
    http_get(&api.connect_server(&server.user_key)).await?;
    
    *state.current_ip.lock().await = Some(ip.clone());
    *state.connection.lock().await = ConnectionState::JoinedServer;

    Ok(())
}

#[tauri::command]
async fn leave_server(
    state: State<'_, AppState>
) -> Result<(), AppError> {
    let server = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", server.0));

    http_get(&api.leave_server()).await?;

    *state.connection.lock().await = ConnectionState::Disconnected;
    *state.session.lock().await = SessionState::Idle;
    
    Ok(())
}

#[tauri::command]
async fn forget_server(
    ip: String, 
    state: State<'_, AppState>
) -> Result<(), AppError> {
    ip.parse::<std::net::SocketAddr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;
    
    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add it first", ip)))?;
    let api = ServerApi::new(&format!("http://{}", ip));
    http_get(&api.forget_server(&server.user_key)).await?;

    state.remove_server(&server.ip).await
        .map_err(|e| AppError::Network(format!("Failed to remove server: {}", e)))?;

    Ok(())
}

#[tauri::command]
async fn set_nickname(
    new_nickname: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    http_get(&api.set_nickname(&server.user_key, &new_nickname)).await?;

    state.save_nickname(&ip, new_nickname).await
        .map_err(|e| AppError::Network(format!("Failed to update nickname: {}", e)))?;
    
    Ok(())
}

// Session commands
#[tauri::command]
async fn get_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<state::Session>, AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));
    
    let response = http_get(&api.session_list(&server.user_key)).await?;
    
    let json_response: state::SessionList = response.json().await
        .map_err(|e| AppError::Protocol(format!("Invalid session list format: {}", e)))?;
    
    Ok(json_response.user_sessions)
}

#[tauri::command]
async fn create_session(
    name: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));
    let response = http_get(&api.create_session(&server.user_key, &name)).await?;
    let json_response: serde_json::Value = response.json().await
        .map_err(|e| AppError::Protocol(format!("Invalid JSON: {}", e)))?;

    let session_key = json_response.get("session_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Protocol("Missing session_key field".into()))?
        .to_string();
    
    Ok(session_key)
}

#[tauri::command]
async fn add_session(
    session_key: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    println!("Attempting to add session '{}' on server at {}", session_key, state.current_ip.lock().await.as_ref().unwrap_or(&"None".to_string()));
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;

    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add a server first", ip)))?;

    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.add_session(&server.user_key, &session_key);
    let client = reqwest::Client::new();
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to add session: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected connection: {}", e)))?;

    Ok(())
}

#[tauri::command]
async fn connect_session(
    session_key: String,
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
    let ws_url = api.ws(&server.user_key, &session_key);
    println!("Connecting to session websocket at {}", ws_url);

    let ws_client = websocket::WsClient::connect(&ws_url, app.clone())
        .await
        .map_err(|e| AppError::Network(format!("Failed to open websocket: {}", e)))?;

    *state.session.lock().await = SessionState::JoinedSession(ws_client);

    let window = app.get_webview_window("main").unwrap();
    window.set_resizable(true).unwrap();
    window.set_maximizable(true).unwrap();
    window.set_size(tauri::Size::Logical(tauri::LogicalSize { width: 1100.0, height: 750.0 })).unwrap();
    Ok(())
}

#[tauri::command]
async fn leave_session(
    state: State<'_, AppState>,   app: AppHandle
) -> Result<(), AppError> {
    println!("Attempting to leave session on server at {}", state.current_ip.lock().await.as_ref().unwrap_or(&"None".to_string()));
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;

    if let SessionState::JoinedSession(ws_client) = &*state.session.lock().await {
        let _ = ws_client.close().await;
    }

    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.leave_session();
    let client = reqwest::Client::new();
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to leave session: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected session leave: {}", e)))?;
    println!("Left session on server at {}", state.current_ip.lock().await.as_ref().unwrap_or(&"None".to_string()));

    *state.session.lock().await = SessionState::Idle;
    let window = app.get_webview_window("main").unwrap();
    window.set_resizable(false).unwrap();
    window.set_maximizable(false).unwrap();
    window.set_size(tauri::Size::Logical(tauri::LogicalSize { width: 360.0, height: 628.0 })).unwrap();
    Ok(())
}

#[tauri::command]
async fn forget_session(
    session_key: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    println!("Attempting to forget session '{}' on server at {}", session_key, state.current_ip.lock().await.as_ref().unwrap_or(&"None".to_string()));
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;

    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add a server first", ip)))?;

    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.forget_session(&server.user_key, &session_key);
    let client = reqwest::Client::new();
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to remove session: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected session removal: {}", e)))?;
    Ok(())
}

#[tauri::command]
async fn delete_session(
    session_key: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    println!("Attempting to delete session '{}' on server at {}", session_key, state.current_ip.lock().await.as_ref().unwrap_or(&"None".to_string()));
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;

    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add a server first", ip)))?;

    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.delete_session(&server.user_key, &session_key);
    let client = reqwest::Client::new();
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to delete session: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected session deletion: {}", e)))?;
    Ok(())
}

#[tauri::command]
async fn send_message(
    body: String, 
    state: State<'_, AppState>
) -> Result<(), AppError> {
    let session = state.session.lock().await;

    if let SessionState::JoinedSession(ws_client) = &*session {
        ws_client.send(&body).await
            .map_err(|e| AppError::Network(format!("Failed to send message: {}", e)))?;
    }
    Ok(())
}

// Client commands
#[tauri::command]
async fn get_servers(
    state: State<'_, AppState>
) -> Result<Vec<state::PersistedServer>, AppError> {
    Ok(state.servers.lock().await.clone())
}

#[tauri::command]
async fn get_nickname(
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;

    let nickname = state.get_nickname(&ip).await
        .ok_or_else(|| AppError::Auth(format!("No nickname found for server {}", ip)))?;

    Ok(nickname)
}

#[tauri::command]
async fn save_nickname(
    nickname: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;

    state.save_nickname(&ip, nickname).await
        .map_err(|e| AppError::Network(format!("Failed to save nickname: {}", e)))?;
    
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
            let state_for_load = app_state.clone(); 
            tauri::async_runtime::block_on(async move {
                state_for_load.load_persisted_data().await;
            });
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Server commands
            add_server,
            connect_server,
            leave_server,
            forget_server,
            set_nickname,
            // Session commands
            get_sessions,
            create_session,
            add_session,
            connect_session,
            leave_session,
            forget_session,
            delete_session,
            send_message,
            // Client commands
            get_servers,
            get_nickname,
            save_nickname,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
