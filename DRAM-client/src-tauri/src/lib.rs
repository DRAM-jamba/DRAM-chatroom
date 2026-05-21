use crate::api::ServerApi;
use crate::error::AppError;
use crate::models::{PersistedServer, Session, SessionKey, SessionList, UserKey, BackMessageObj, MessageType};
use crate::state::{AppState, ServerConnectionState, SessionState};

use tauri::{AppHandle, Manager, State};

mod api;
mod client;
mod error;
mod events;
mod models;
mod security;
mod state;

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

// Server commands
#[tauri::command]
async fn add_server(
    ip: String,
    nickname: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    ip.parse::<std::net::SocketAddr>()
        .map_err(|_| AppError::Network(format!("Invalid IP address: '{}'", ip)))?;

    let api = ServerApi::new(&format!("http://{}", ip));
    let response = ServerApi::http_post_empty(&api.add_server()).await?;
    let json_response: UserKey = response
        .json()
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to parse user key: {}", e)))?;

    state.set_connection_ip(ip.clone()).await;
    state
        .set_connection_state(ServerConnectionState::Connected)
        .await;

    state
        .add_server(ip, nickname, json_response.user_key)
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

    ServerApi::http_put(
        &api.connect_server(),
        &serde_json::json!({ "user_key": server.user_key }),
    )
    .await?;

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

    ServerApi::http_delete_empty(&api.leave_server()).await?;

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

    ServerApi::http_delete(
        &api.forget_server(),
        &serde_json::json!({ "user_key": server.user_key }),
    )
    .await?;

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

    ServerApi::http_patch(
        &api.set_nickname(),
        &serde_json::json!({ "user_key": server.user_key, "nickname": new_nickname }),
    )
    .await?;

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

    let response = ServerApi::http_get(
        &api.session_list(),
        &serde_json::json!({ "user_key": server.user_key }),
    )
    .await?;

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

    let response = ServerApi::http_post(
        &api.create_session(),
        &serde_json::json!({ "user_key": server.user_key, "session_name": name }),
    )
    .await?;

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

    ServerApi::http_post(
        &api.add_session(),
        &serde_json::json!({ "user_key": server.user_key, "session_key": session_key }),
    )
    .await?;

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

    let ws_url = api.ws();
    let ws_client = client::WsClient::connect(&ws_url, &server.user_key, &session_key, app.clone())
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

    ServerApi::http_delete_empty(&api.leave_session()).await?;

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

    ServerApi::http_delete(
        &api.forget_session(),
        &serde_json::json!({ "user_key": server.user_key, "session_key": session_key }),
    )
    .await?;

    Ok(())
}

#[tauri::command]
async fn delete_session(session_key: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));

    ServerApi::http_delete(
        &api.delete_session(),
        &serde_json::json!({ "user_key": server.user_key, "session_key": session_key }),
    )
    .await?;

    Ok(())
}

#[tauri::command]
async fn send_message(body: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let session = state.get_session_state();

    if let SessionState::JoinedSession(ws_client) = &session.await {
        let payload = BackMessageObj {
            m_type: MessageType::Message,
            body,
        };

        let serialized_body = serde_json::to_string(&payload)
            .map_err(|e| AppError::Network(format!("Failed to serialize message: {}", e)))?;
        ws_client
            .send(&serialized_body)
            .await
            .map_err(|e| AppError::Network(format!("Failed to send message: {}", e)))?; 
    }
    Ok(())
}

// Voice-chat

#[tauri::command]
async fn reset_mic_permission(app: AppHandle) -> Result<(), AppError> {
    let data_dir = app.path().app_local_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get data dir: {}", e)))?;
    
    let prefs_path = data_dir
        .join("EBWebView")
        .join("Default")
        .join("Preferences");
    
    let prefs_str = prefs_path.to_string_lossy().to_string();

    std::process::Command::new("powershell")
        .args([
            "-Command",
            &format!("Start-Sleep -Seconds 2; Remove-Item -Force '{}'", prefs_str)
        ])
        .spawn()
        .map_err(|e| AppError::Internal(format!("Failed to spawn cleanup: {}", e)))?;
    
    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn join_voice_chat(session_key: String, state: State<'_, AppState>) -> Result<models::VoiceChatInfo, AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let api = ServerApi::new(&format!("http://{}", ip));
 
    let response = ServerApi::http_get(
        &api.create_voicechat(),
        &serde_json::json!({ "user_key": server.user_key, "session_key": session_key }),
    )
    .await?;
 
    let json_response: models::VoiceToken = response
        .json()
        .await
        .map_err(|e| AppError::Protocol(format!("Invalid voice token response: {}", e)))?;
    
    let host = ip.split(':').next().unwrap_or(&ip);
    let lk_url = format!("ws://{}:7880", host);
 
    Ok(models::VoiceChatInfo {
        token: json_response.token,
        url: lk_url,
    })
}

#[tauri::command]
async fn send_voice_signal(m_type: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let msg_type = match m_type.as_str() {
        "voicestart" => MessageType::VoiceStart,
        "voiceend" => MessageType::VoiceEnd,
        _ => return Err(AppError::Protocol(format!("Invalid voice signal type: {}", m_type))),
    };
 
    let session = state.get_session_state();
    if let SessionState::JoinedSession(ws_client) = &session.await {
        let payload = BackMessageObj {
            m_type: msg_type,
            body: String::new(),
        };
        let serialized = serde_json::to_string(&payload)
            .map_err(|e| AppError::Network(format!("Failed to serialize voice signal: {}", e)))?;
        ws_client
            .send(&serialized)
            .await
            .map_err(|e| AppError::Network(format!("Failed to send voice signal: {}", e)))?;
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


pub fn run() {
    let master_key =
        security::get_or_create_master_key().expect("Failed to initialize secure system storage");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let app_state = AppState::new(app_handle.clone(), master_key);
            app.manage(app_state.clone());

            tauri::async_runtime::spawn(async move {
                if let Err(e) = app_state.load_persisted_data().await {
                    eprintln!("Data Recovery Notice: {}", e);
                }
            });

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
            //Voice chat commands
            reset_mic_permission,
            join_voice_chat,
            send_voice_signal,
            // Client commands
            get_servers,
            get_nickname,
            save_nickname,
        ])
        .run(tauri::generate_context!())
        .expect("error while running quorthon");
}
