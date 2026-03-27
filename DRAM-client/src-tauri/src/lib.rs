use crate::error::AppError;
use crate::state::{AppState, ConnectionState, SessionState};
use crate::websocket::WsClient;
use tauri::{AppHandle, State};

mod error;
mod events;
mod state;
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

    let client = reqwest::Client::new();
    let url = format!("http://{}/add", ip);
    
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
    let user_key = state.current_user_key.lock().await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::Auth("No user key — use add first".into()))?;
    let client = reqwest::Client::new();
    let url = format!("http://{}/connect/{}", ip, user_key);
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Failed to connect: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Auth(format!("Server rejected connection: {}", e)))?;

    // Emit event to frontend | will be used to change UI state
    // events::emit_connected(&app);
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
async fn join_session(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let conn = state.connection.lock().await;
    if let ConnectionState::Connected(client) = &*conn {
        let msg = serde_json::json!({
            "action": "join",
            "session_id": session_id
        });
        client.send(&msg.to_string()).await?;
        *state.session.lock().await = SessionState::JoinedSession {
            session_id: session_id.clone(),
            participants: vec![],
        };
        events::emit_session_update(
            &app,
            events::SessionPayload {
                session_id,
                participants: vec![],
                chat_log: vec![],
            },
        );
        Ok(())
    } else {
        Err(AppError::Network("Not connected".into()))
    }
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
