use crate::api::ServerApi;
use crate::error::AppError;
use crate::models::{PersistedServer, Session, SessionKey, SessionList, UserKey};
use crate::state::{AppState, ServerConnectionState, SessionState};

use tauri::{AppHandle, Manager, State};

mod api;
mod client;
mod error;
mod events;
mod models;
mod state;
mod security;

// Helper functions
async fn get_server_context(
    state: &State<'_, AppState>,
) -> Result<(String, models::PersistedServer), AppError> {
    let ip = state
        .get_current_ip()
        .await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Session("Not connected to server".into()))?;

    let server = state
        .get_server(&ip)
        .await
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
async fn add_server(ip: String, state: State<'_, AppState>) -> Result<(), AppError> {
    ip.parse::<std::net::SocketAddr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;

    let api = ServerApi::new(&format!("http://{}", ip));
    let response = http_get(&api.add_server()).await?;
    let json_response: UserKey = response
        .json()
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to parse user key: {}", e)))?;

    let temp_nick = "".to_string();

    state.set_connection_ip(ip.clone()).await;
    state
        .set_connection_state(ServerConnectionState::Connected)
        .await;

    state
        .add_server(ip, temp_nick, json_response.user_key)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save: {}", e)))?;

    Ok(())
}

#[tauri::command]
async fn connect_server(ip: String, state: State<'_, AppState>) -> Result<(), AppError> {
    ip.parse::<std::net::SocketAddr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;

    let server = state
        .get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add it first", ip)))?;
    let api = ServerApi::new(&format!("http://{}", ip));

    http_get(&api.connect_server(&server.user_key)).await?;

    state.set_connection_ip(ip).await;
    state
        .set_connection_state(ServerConnectionState::Connected)
        .await;

    Ok(())
}

#[tauri::command]
async fn leave_server(state: State<'_, AppState>) -> Result<(), AppError> {
    let (ip, _server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    http_get(&api.leave_server()).await?;
    let empty_ip = "0.0.0.0:0000";

    state.set_connection_ip(empty_ip.to_string()).await;
    state
        .set_connection_state(ServerConnectionState::Disconnected)
        .await;
    state.set_session_state(SessionState::Idle).await;

    Ok(())
}

#[tauri::command]
async fn forget_server(ip: String, state: State<'_, AppState>) -> Result<(), AppError> {
    ip.parse::<std::net::SocketAddr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;

    let server = state
        .get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add it first", ip)))?;
    let api = ServerApi::new(&format!("http://{}", ip));
    http_get(&api.forget_server(&server.user_key)).await?;

    state
        .remove_server(&server.ip)
        .await
        .map_err(|e| AppError::Network(format!("Failed to remove server: {}", e)))?;

    Ok(())
}

#[tauri::command]
async fn set_nickname(new_nickname: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    http_get(&api.set_nickname(&server.user_key, &new_nickname)).await?;

    state
        .save_nickname(&ip, new_nickname)
        .await
        .map_err(|e| AppError::Network(format!("Failed to update nickname: {}", e)))?;

    Ok(())
}

// Session commands
#[tauri::command]
async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    let response = http_get(&api.session_list(&server.user_key)).await?;

    let json_response: SessionList = response
        .json()
        .await
        .map_err(|e| AppError::Protocol(format!("Invalid session list format: {}", e)))?;

    Ok(json_response.user_sessions)
}

#[tauri::command]
async fn create_session(name: String, state: State<'_, AppState>) -> Result<String, AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    let response = http_get(&api.create_session(&server.user_key, &name)).await?;

    let json_response: SessionKey = response
        .json()
        .await
        .map_err(|e| AppError::Protocol(format!("Invalid response: {}", e)))?;

    Ok(json_response.session_key)
}

#[tauri::command]
async fn add_session(session_key: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    http_get(&api.add_session(&server.user_key, &session_key)).await?;

    Ok(())
}

#[tauri::command]
async fn connect_session(
    session_key: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    let ws_url = api.ws(&server.user_key, &session_key);
    let ws_client = client::WsClient::connect(&ws_url, app.clone())
        .await
        .map_err(|e| AppError::Network(format!("Failed to open websocket: {}", e)))?;

    state
        .set_session_state(SessionState::JoinedSession(ws_client))
        .await;

    let window = app.get_webview_window("main").unwrap();
    window.set_resizable(true).unwrap();
    window.set_maximizable(true).unwrap();
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 1100.0,
            height: 750.0,
        }))
        .unwrap();
    Ok(())
}

#[tauri::command]
async fn leave_session(state: State<'_, AppState>, app: AppHandle) -> Result<(), AppError> {
    let (ip, _server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    if let SessionState::JoinedSession(ws_client) = &state.get_session_state().await {
        let _ = ws_client.close().await;
    }

    http_get(&api.leave_session()).await?;

    state.set_session_state(SessionState::Idle).await;

    let window = app.get_webview_window("main").unwrap();
    window.set_resizable(false).unwrap();
    window.set_maximizable(false).unwrap();
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 360.0,
            height: 628.0,
        }))
        .unwrap();
    Ok(())
}

#[tauri::command]
async fn forget_session(session_key: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    http_get(&api.forget_session(&server.user_key, &session_key)).await?;

    Ok(())
}

#[tauri::command]
async fn delete_session(session_key: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    http_get(&api.delete_session(&server.user_key, &session_key)).await?;

    Ok(())
}

#[tauri::command]
async fn send_message(body: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let session = state.get_session_state();

    if let SessionState::JoinedSession(ws_client) = &session.await {
        ws_client
            .send(&body)
            .await
            .map_err(|e| AppError::Network(format!("Failed to send message: {}", e)))?;
    }
    Ok(())
}

// Client commands
#[tauri::command]
async fn get_servers(state: State<'_, AppState>) -> Result<Vec<PersistedServer>, AppError> {
    Ok(state.get_all_servers().await)
}

#[tauri::command]
async fn get_nickname(state: State<'_, AppState>) -> Result<String, AppError> {
    let (ip, _server) = get_server_context(&state).await?;

    let nickname = state
        .get_nickname(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No nickname found for server {}", ip)))?;

    Ok(nickname)
}

#[tauri::command]
async fn save_nickname(nickname: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let (ip, _server) = get_server_context(&state).await?;

    state
        .save_nickname(&ip, nickname)
        .await
        .map_err(|e| AppError::Network(format!("Failed to save nickname: {}", e)))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let master_key = security::get_or_create_master_key()
        .expect("Failed to initialize secure system storage");

    tauri::Builder::default()
        .plugin(tauri_plugin_stronghold::Builder::new(move |_password| {
            master_key.to_vec()
        }).build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_state = AppState::new(app_handle);

            let state_for_load = app_state.clone();
            tauri::async_runtime::block_on(async move {
                if let Err(e) = state_for_load.load_persisted_data().await {
                    eprintln!("Data Recovery Notice: Starting with fresh state. Details: {}", e);
                }
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
        .expect("error while running quorthon");
}
