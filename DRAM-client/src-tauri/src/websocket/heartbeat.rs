use std::time::Duration;
use tokio::time::interval;
use crate::state::{AppState, ConnectionState};

pub async fn start(app_state: &AppState) {
    let connection = app_state.connection.clone();  // ← field on the struct, not module-level
    let heartbeat  = app_state.heartbeat.clone();   // ← same

    let handle = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;                    // ← no underscores
            let conn = connection.lock().await;
            if let ConnectionState::Connected(client) = &*conn {
                if client.ping().await.is_err() {
                    break;
                }
            } else {
                break;
            }
        }
    });

    *heartbeat.lock().await = Some(handle);
}

pub async fn stop(app_state: &AppState) {           // ← &AppState not AppState
    if let Some(handle) = app_state.heartbeat       // ← lowercase, field access
        .lock().await
        .take()
    {
        handle.abort();
    }
}