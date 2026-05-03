use crate::error::AppError;
use crate::events::{emit_joined_session, emit_session_update};
use crate::state::{AppState, ConnectionState, SessionState};
use crate::api::ServerApi;
use tauri::{AppHandle, State, ipc};
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

// Server commands
#[tauri::command]
async fn add_server(
    ip: String,
    nickname: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    ip.parse::<std::net::SocketAddr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;

    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.add_server();

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

    state.add_server(ip.clone(), nickname, user_key)
        .await
        .map_err(|e| AppError::Network(format!("Failed to save server: {}", e)))?;
    events::emit_connected(&app);
    Ok(())
}

#[tauri::command]
async fn connect_server(
    ip: String,
    nickname: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    ip.parse::<std::net::SocketAddr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;
    println!("Attempting to connect to server at {}", ip);
    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — use add first", ip)))?;
    
    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.connect_server(&server.user_key);
    let client = reqwest::Client::new();
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to connect: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected connection: {}", e)))?;

    *state.current_ip.lock().await = Some(ip.clone());
    println!("State ip updated to {}", state.current_ip.lock().await.as_ref().unwrap());
    *state.connection.lock().await = ConnectionState::JoinedServer;

    if let Some(nick) = nickname {
        let url = api.set_nickname(&server.user_key, &nick);
        let client = reqwest::Client::new();
        client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to set nickname: {}", e)))?
            .error_for_status()
            .map_err(|e| AppError::Auth(format!("Server rejected nickname change: {}", e)))?;
    }

    events::emit_connected(&app);
    Ok(())
}

#[tauri::command]
async fn leave_server(
    state: State<'_, AppState>
) -> Result<(), AppError> {
    println!("Attempting to leave server at {}", state.current_ip.lock().await.as_ref().unwrap_or(&"None".to_string()));
    let ip = state.current_ip.lock().await
        .take()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;
    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.leave_server();
    let client = reqwest::Client::new();
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to disconnect: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected disconnect: {}", e)))?;

    *state.connection.lock().await = ConnectionState::Disconnected;
    *state.session.lock().await = SessionState::Idle;
    println!("Left server");
    Ok(())
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

        state.save_nickname(&ip, new_nickname).await
            .map_err(|e| AppError::Network(format!("Failed to update nickname: {}", e)))?;
    
    Ok(())
}

#[tauri::command]
async fn forget_server(
    id: String, 
    state: State<'_, AppState>
) -> Result<(), AppError> {
    println!("Attempting to forget server with ID '{}'", &id);
    let server = state.get_server_by_id(&id)
        .await
        .ok_or_else(|| AppError::Auth(format!("Server does not exist: {}", &id)))?;

    let api = ServerApi::new(&format!("http://{}", server.ip));
    let url = api.forget_server(&server.user_key);
    let client = reqwest::Client::new();
    let _ = client.get(&url).send().await;

    state.remove_server(&server.ip).await
        .map_err(|e| AppError::Network(format!("Failed to remove server: {}", e)))?;

    Ok(())
}

// Session commands
#[tauri::command]
async fn get_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<state::Session>, AppError> {
    println!("Attempting to retrieve sessions on server at {}", state.current_ip.lock().await.as_ref().unwrap_or(&"None".to_string()));
    let ip = state.current_ip.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Network("Not connected to server".into()))?;

    let server = state.get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add a server first", ip)))?;

    let api = ServerApi::new(&format!("http://{}", ip));
    let url = api.session_list(&server.user_key);
    let client = reqwest::Client::new();

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to retrieve sessions: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected session list request: {}", e)))?;

    let response_obj: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Network(format!("Failed to parse session list: {}", e)))?;

    let sessions: Vec<serde_json::Value> = response_obj
        .get("related_sessions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let session_list = sessions
        .into_iter()
        .map(|session| {
            let session_key = session.get("session_key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = session.get("session_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed")
                .to_string();
            
            state::Session {
                id: session_key,
                name,
                last_connected: "now".to_string(),
            }
        })
        .collect();

    Ok(session_list)
}

#[tauri::command]
async fn create_session(
    name: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    println!("Attempting to create session '{}' on server at {}", name, state.current_ip.lock().await.as_ref().unwrap_or(&"None".to_string()));
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

    let session_key: String = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Network(format!("Failed to parse response: {}", e)))?
        .get("session_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Network("Missing session_key in response".into()))?
        .to_string();
    println!("Session created with key {}", session_key);
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

    emit_joined_session(&app);
    Ok(())
}

#[tauri::command]
async fn leave_session(
    state: State<'_, AppState>
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
    // TODO: implement
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
            greet,
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
