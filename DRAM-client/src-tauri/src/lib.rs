use crate::api::ServerApi;
use crate::error::AppError;
use crate::models::{
    BackMessageObj, ChallengeSolvePayload, MessageType, PersistedServer, Session, SessionKey,
    SessionList, UserKey,
};
use crate::security::get_public_key_hex;
use crate::state::{AppState, ServerConnectionState, SessionState};
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

pub mod api;
mod client;
mod error;
mod events;
pub mod models;
mod security;
mod state;
mod hotkeys;

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

async fn solve_challenge(
    master_key: &[u8; 32],
    challenge_hex: String,
    user_key: String,
) -> Result<ChallengeSolvePayload, String> {
    use ed25519_dalek::Signer; 
    let signing_key = security::derive_identity_keypair(master_key).0;
    let signature = signing_key.sign(challenge_hex.as_bytes());
    Ok(ChallengeSolvePayload {
        nonce: challenge_hex,
        signature: hex::encode(signature.to_bytes()),
        user_key,
    })
}

fn start_token_refresh_worker(state: tauri::State<'_, AppState>) {
    let state_clone = state.inner().clone();
    println!("Starting token refresh worker...");
    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(7 * 60));
        interval.tick().await;

        loop {
            interval.tick().await;

            let current_ip = state_clone.get_current_ip().await;
            let current_token = state_clone.get_token().await;
            println!(
                "[Worker] Tick: Current IP: {:?}, Current Token: {:?}",
                current_ip, current_token
            );
            println!("[Worker] Attempting to refresh token...");
            if let (Some(ip), Some(token)) = (current_ip, current_token) {
                if let Some(server) = state_clone.get_server(&ip).await {
                    let api = crate::api::ServerApi::with_token(&format!("https://{}", ip), token);
                    let payload = serde_json::json!({ "user_key": server.user_key });

                    match api
                        .http_patch_authed(&api.refresh_token_url(), &payload)
                        .await
                    {
                        Ok(response) => {
                            #[derive(serde::Deserialize)]
                            struct TokenResponse {
                                token: String,
                            }
                            if let Ok(json_res) = response.json::<TokenResponse>().await {
                                state_clone.set_token(json_res.token).await;
                                println!("[Worker] Token refreshed successfully");
                            }
                        }
                        Err(e) => {
                            eprintln!("[Worker] Refresh failed: {:?}", e);
                            // TODO: Proper error handling - log out the user and stop the worker
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }
    });

    let state_for_setter = state.inner().clone();
    tokio::spawn(async move {
        state_for_setter.set_refresh_task(Some(handle)).await;
    });
}

// Server commands
#[tauri::command]
async fn add_server(
    ip: String,
    nickname: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let api = ServerApi::new(&format!("https://{}", ip));
    let public_key = get_public_key_hex(&*state.get_master_key());
    println!(
        "Adding server with IP: {}, using public key: {}",
        ip, public_key
    );
    let response = ServerApi::http_post(
        &api.add_server(),
        &serde_json::json!({ "public_key": public_key }),
    )
    .await?;

    let json_response: UserKey = response
        .json()
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to parse user key: {}", e)))?;

    state
        .add_server(ip, nickname, json_response.user_key)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save: {}", e)))?;

    Ok(())
}

#[tauri::command]
async fn connect_server(ip: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let server = state
        .get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add it first", ip)))?;
    let api = ServerApi::new(&format!("https://{}", ip));

    let challenge_response = ServerApi::http_get(
        &api.challenge(),
        &serde_json::json!({ "user_key": server.user_key }),
    )
    .await?;

    let challenge_data: models::ChallengeFromServer = challenge_response
        .json()
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to parse challenge: {}", e)))?;
    
    let challenge_payload = solve_challenge(
        &*state.get_master_key(),
        challenge_data.challenge,
        server.user_key,
    )
    .await
    .map_err(|e| AppError::Auth(e))?;

    let token_response = ServerApi::http_post(&api.token_url(), &challenge_payload).await?;
    let token_data: models::TokenResponse = token_response
        .json()
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to parse token: {}", e)))?;

    state.set_token(token_data.token).await;
    state.set_connection_ip(ip).await;
    state.set_connection_state(ServerConnectionState::Connected).await;

    start_token_refresh_worker(state.clone());

    Ok(())
}

#[tauri::command]
async fn leave_server(state: State<'_, AppState>) -> Result<(), AppError> {
    let (_ip, _server) = get_server_context(&state).await?;

    state.set_refresh_task(None).await;
    state.clear_token().await;

    let empty_ip = "0.0.0.0:0000";
    state.set_connection_ip(empty_ip.to_string()).await;
    state.set_connection_state(ServerConnectionState::Disconnected).await;
    state.set_session_state(SessionState::Idle).await;

    Ok(())
}

#[tauri::command]
async fn forget_server(ip: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let server = state
        .get_server(&ip)
        .await
        .ok_or_else(|| AppError::Auth(format!("No user key for {} — add it first", ip)))?;

    let api = ServerApi::new(&format!("https://{}", ip));
    let challenge_response = ServerApi::http_get(
        &api.challenge(),
        &serde_json::json!({ "user_key": server.user_key }),
    ).await?;

    let challenge_data: models::ChallengeFromServer = challenge_response.json().await?;
    let challenge_payload = solve_challenge(
        &*state.get_master_key(),
        challenge_data.challenge,
        server.user_key.clone(),
    )
    .await
    .map_err(|e| AppError::Auth(e))?;

    let token_response = ServerApi::http_post(&api.token_url(), &challenge_payload).await?;
    let token_data: models::TokenResponse = token_response.json().await?;

    let api_with_token = ServerApi::with_token(&format!("https://{}", ip), token_data.token);
    api_with_token
        .http_delete_authed(
            &api_with_token.forget_server(),
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
    let token = state.get_token().await.ok_or_else(|| {
        AppError::Auth("No authentication token. Connect to server first.".into())
    })?;

    let api = ServerApi::with_token(&format!("https://{}", ip), token);
    api.http_patch_authed(
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

#[tauri::command]
async fn rename_server(
    ip: String,
    nickname: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state
        .rename_server(&ip, nickname)
        .await
        .map_err(|e| AppError::Network(format!("Failed to rename server: {}", e)))?;

    Ok(())
}

// Session commands
#[tauri::command]
async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, AppError> {
    let (ip, server) = get_server_context(&state).await?;

    let token = state.get_token().await.ok_or_else(|| {
        AppError::Auth("No authentication token. Connect to server first.".into())
    })?;

    let api = ServerApi::with_token(&format!("https://{}", ip), token);

    let response = api
        .http_get_authed(
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

    let token = state.get_token().await.ok_or_else(|| {
        AppError::Auth("No authentication token. Connect to server first.".into())
    })?;

    let api = ServerApi::with_token(&format!("https://{}", ip), token);

    let response = api
        .http_post_authed(
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

    let token = state.get_token().await.ok_or_else(|| {
        AppError::Auth("No authentication token. Connect to server first.".into())
    })?;

    let api = ServerApi::with_token(&format!("https://{}", ip), token);

    api.http_post_authed(
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
    let api = ServerApi::new(&format!("https://{}", ip));

    let ws_url = api.ws();
    let ws_client = client::WsClient::connect(&ws_url, &server.user_key, &session_key, app.clone())
        .await
        .map_err(|e| AppError::Network(format!("Failed to open websocket: {}", e)))?;

    state
        .set_session_state(SessionState::JoinedSession(ws_client))
        .await;
    Ok(())
}

#[tauri::command]
async fn resize_for_chat(app: AppHandle) -> Result<(), AppError> {
    let window = app.get_webview_window("main").unwrap();
    window.set_resizable(true).unwrap();
    window.set_maximizable(true).unwrap();
    window
        .set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize {
            width: 1100.0,
            height: 740.0,
        })))
        .unwrap();
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 1100.0,
            height: 750.0,
        }))
        .unwrap();
    Ok(())
}

#[tauri::command]
async fn resize_for_sessions(app: AppHandle) -> Result<(), AppError> {
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
async fn leave_session(state: State<'_, AppState>, _app: AppHandle) -> Result<(), AppError> {
    let (ip, server) = get_server_context(&state).await?;

    let token = state.get_token().await.ok_or_else(|| {
        AppError::Auth("No authentication token. Connect to server first.".into())
    })?;

    let api = ServerApi::with_token(&format!("https://{}", ip), token);

    if let SessionState::JoinedSession(ws_client) = &state.get_session_state().await {
        let _ = ws_client.close().await;
    }

    api.http_delete_authed(
        &api.leave_session(),
        &serde_json::json!({ "user_key": server.user_key }),
    )
    .await?;

    state.set_session_state(SessionState::Idle).await;
    Ok(())
}

#[tauri::command]
async fn forget_session(session_key: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let (ip, server) = get_server_context(&state).await?;

    let token = state.get_token().await.ok_or_else(|| {
        AppError::Auth("No authentication token. Connect to server first.".into())
    })?;

    let api = ServerApi::with_token(&format!("https://{}", ip), token);

    api.http_delete_authed(
        &api.forget_session(),
        &serde_json::json!({ "user_key": server.user_key, "session_key": session_key }),
    )
    .await?;

    Ok(())
}

#[tauri::command]
async fn delete_session(session_key: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let (ip, server) = get_server_context(&state).await?;

    let token = state.get_token().await.ok_or_else(|| {
        AppError::Auth("No authentication token. Connect to server first.".into())
    })?;

    let api = ServerApi::with_token(&format!("https://{}", ip), token);

    api.http_delete_authed(
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
    let data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to get data dir: {}", e)))?;

    let prefs_path = data_dir
        .join("EBWebView")
        .join("Default")
        .join("Preferences");

    let prefs_str = prefs_path.to_string_lossy().to_string();

    std::process::Command::new("powershell")
        .args([
            "-Command",
            &format!("Start-Sleep -Seconds 2; Remove-Item -Force '{}'", prefs_str),
        ])
        .spawn()
        .map_err(|e| AppError::Internal(format!("Failed to spawn cleanup: {}", e)))?;

    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn join_voice_chat(
    session_key: String,
    state: State<'_, AppState>,
) -> Result<models::VoiceChatInfo, AppError> {
    let (ip, server) = get_server_context(&state).await?;
    let host = ip.split(':').next().unwrap_or(&ip);
    let lk_url = format!("ws://{}:7880", host);

    let token = state.get_token().await.ok_or_else(|| {
        AppError::Auth("No authentication token".into())
    })?;

    let api = ServerApi::with_token(&format!("https://{}", ip), token);

    let response = api
        .http_get_authed(
            &api.create_voicechat(),
            &serde_json::json!({ "user_key": server.user_key, "session_key": session_key }),
        )
        .await?;

    let json_response: models::VoiceToken = response
        .json()
        .await
        .map_err(|e| AppError::Protocol(format!("Invalid voice token: {}", e)))?;

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
        _ => {
            return Err(AppError::Protocol(format!(
                "Invalid voice signal type: {}",
                m_type
            )))
        }
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




#[tauri::command]
async fn register_hotkeys(mic_key: String, headphones_key: String, app: AppHandle) -> Result<(), AppError> {
    hotkeys::unregister_hooks();
    let mic_vk = if mic_key.is_empty() { None } else { hotkeys::key_str_to_vk(&mic_key) };
    let hp_vk = if headphones_key.is_empty() { None } else { hotkeys::key_str_to_vk(&headphones_key) };
    hotkeys::register_hooks(app, mic_vk, hp_vk);
    Ok(())
}

#[tauri::command]
async fn unregister_hotkeys() -> Result<(), AppError> {
    hotkeys::unregister_hooks();
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
            rename_server,
            // Session commands
            get_sessions,
            create_session,
            add_session,
            connect_session,
            leave_session,
            forget_session,
            delete_session,
            send_message,
            resize_for_chat,
            resize_for_sessions,
            //Voice chat commands
            reset_mic_permission,
            join_voice_chat,
            send_voice_signal,
            // Client commands
            get_servers,
            get_nickname,
            save_nickname,
            // mic hotkeys
            register_hotkeys,
            unregister_hotkeys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running quorthon");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PersistedServer, Session};

    #[allow(dead_code)]
    struct MockAppState {
        current_ip: Option<String>,
        servers: Vec<PersistedServer>,
        nicknames: std::collections::HashMap<String, String>,
    }

    impl MockAppState {
        fn new() -> Self {
            MockAppState {
                current_ip: None,
                servers: Vec::new(),
                nicknames: std::collections::HashMap::new(),
            }
        }

        fn with_servers(mut self, servers: Vec<PersistedServer>) -> Self {
            self.servers = servers;
            self
        }

        fn with_current_ip(mut self, ip: String) -> Self {
            self.current_ip = Some(ip);
            self
        }

        fn with_nickname(mut self, ip: String, nickname: String) -> Self {
            self.nicknames.insert(ip, nickname);
            self
        }

        async fn get_all_servers(&self) -> Vec<PersistedServer> {
            self.servers.clone()
        }

        #[allow(dead_code)]
        async fn get_current_ip(&self) -> Option<String> {
            self.current_ip.clone()
        }

        async fn get_nickname(&self, ip: &str) -> Option<String> {
            self.nicknames.get(ip).cloned()
        }
    }

    #[test]
    fn test_get_servers_empty() {
        let state = MockAppState::new();
        let servers = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(state.get_all_servers())
        })
        .join()
        .unwrap();

        assert_eq!(servers.len(), 0);
    }

    #[test]
    fn test_get_servers_with_data() {
        let test_servers = vec![
            PersistedServer {
                id: "server1".to_string(),
                ip: "127.0.0.1:8080".to_string(),
                server_name: "Test Server".to_string(),
                user_key: "key123".to_string(),
                user_nickname: Some("TestUser".to_string()),
            },
            PersistedServer {
                id: "server2".to_string(),
                ip: "192.168.1.1:8080".to_string(),
                server_name: "Production Server".to_string(),
                user_key: "key456".to_string(),
                user_nickname: None,
            },
        ];

        let state = MockAppState::new().with_servers(test_servers.clone());
        let servers = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(state.get_all_servers())
        })
        .join()
        .unwrap();

        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].ip, "127.0.0.1:8080");
        assert_eq!(servers[1].ip, "192.168.1.1:8080");
        assert_eq!(servers[0].user_nickname, Some("TestUser".to_string()));
        assert_eq!(servers[1].user_nickname, None);
    }

    #[test]
    fn test_get_sessions_parsing() {
        let json_str = r#"
        {
            "user_sessions": [
                {
                    "session_key": "session_abc123",
                    "session_name": "General Chat",
                    "user_role": "member"
                },
                {
                    "session_key": "session_def456",
                    "session_name": "Private Group",
                    "user_role": "admin"
                }
            ]
        }
        "#;

        let session_list: models::SessionList =
            serde_json::from_str(json_str).expect("Failed to parse SessionList");

        assert_eq!(session_list.user_sessions.len(), 2);
        assert_eq!(session_list.user_sessions[0].id, "session_abc123");
        assert_eq!(session_list.user_sessions[0].name, "General Chat");
        assert_eq!(session_list.user_sessions[0].user_role, "member");
        assert_eq!(session_list.user_sessions[1].user_role, "admin");
    }

    #[test]
    fn test_get_sessions_empty() {
        let json_str = r#"
        {
            "user_sessions": []
        }
        "#;

        let session_list: models::SessionList =
            serde_json::from_str(json_str).expect("Failed to parse empty SessionList");

        assert_eq!(session_list.user_sessions.len(), 0);
    }

    #[test]
    fn test_create_session_key_extraction() {
        // Test that SessionKey deserializes and extracts correctly
        let json_str = r#"
        {
            "session_key": "new_session_xyz789"
        }
        "#;

        let session_key: models::SessionKey =
            serde_json::from_str(json_str).expect("Failed to parse SessionKey");

        assert_eq!(session_key.session_key, "new_session_xyz789");
    }

    #[test]
    fn test_get_nickname_exists() {
        let state = MockAppState::new()
            .with_current_ip("127.0.0.1:8080".to_string())
            .with_nickname("127.0.0.1:8080".to_string(), "TestUser".to_string());

        let nickname = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(state.get_nickname("127.0.0.1:8080"))
        })
        .join()
        .unwrap();

        assert_eq!(nickname, Some("TestUser".to_string()));
    }

    #[test]
    fn test_get_nickname_not_found() {
        // Test that None is returned when nickname doesn't exist
        let state = MockAppState::new();

        let nickname = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(state.get_nickname("unknown_ip"))
        })
        .join()
        .unwrap();

        assert_eq!(nickname, None);
    }

    #[test]
    fn test_join_voice_chat_url_construction() {
        // Test that voice chat URL is constructed correctly from IP
        let test_cases = vec![
            ("127.0.0.1:8080", "wss://127.0.0.1:7880"),
            ("192.168.1.100:9000", "wss://192.168.1.100:7880"),
            ("localhost:8000", "wss://localhost:7880"),
        ];

        for (ip, expected_url) in test_cases {
            let lk_url = format!("wss://{}", ip);
            assert_eq!(lk_url, expected_url);
        }
    }

    #[test]
    fn test_voice_token_deserialization() {
        let json_str = r#"
        {
            "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"
        }
        "#;

        let token: models::VoiceToken =
            serde_json::from_str(json_str).expect("Failed to parse VoiceToken");

        assert_eq!(token.token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_user_key_deserialization() {
        // Test that UserKey deserializes correctly
        let json_str = r#"
        {
            "user_key": "user_abc123def456"
        }
        "#;

        let user_key: models::UserKey =
            serde_json::from_str(json_str).expect("Failed to parse UserKey");

        assert_eq!(user_key.user_key, "user_abc123def456");
    }

    #[test]
    fn test_message_serialization() {
        // Test that BackMessageObj serializes correctly
        let msg = models::BackMessageObj {
            m_type: models::MessageType::Message,
            body: "Hello, World!".to_string(),
        };

        let json = serde_json::to_string(&msg).expect("Failed to serialize message");
        assert!(json.contains("Hello, World!"));
        assert!(json.contains("message") || json.contains("Message"));
    }

    #[test]
    fn test_persisted_server_structure() {
        // Test that PersistedServer can be created and has correct fields
        let server = PersistedServer {
            id: "test_id".to_string(),
            ip: "10.0.0.1:8080".to_string(),
            server_name: "Test".to_string(),
            user_key: "key123".to_string(),
            user_nickname: Some("Nick".to_string()),
        };

        assert_eq!(server.id, "test_id");
        assert_eq!(server.ip, "10.0.0.1:8080");
        assert_eq!(server.server_name, "Test");
        assert_eq!(server.user_key, "key123");
        assert_eq!(server.user_nickname, Some("Nick".to_string()));
    }

    #[test]
    fn test_session_structure() {
        let session = Session {
            id: "session_1".to_string(),
            name: "Test Session".to_string(),
            user_role: "admin".to_string(),
        };

        assert_eq!(session.id, "session_1");
        assert_eq!(session.name, "Test Session");
        assert_eq!(session.user_role, "admin");
    }
}
