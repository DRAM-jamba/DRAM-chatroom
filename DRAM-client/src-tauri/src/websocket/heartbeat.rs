use crate::state::{AppState, ConnectionState};
use std::time::Duration;
use tokio::time::interval;

pub async fn start(app_state: &AppState) {
    let connection = app_state.connection.clone();
    let heartbeat = app_state.heartbeat.clone();

    let handle = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await; // ← no underscores
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

pub async fn stop(app_state: &AppState) {
    // ← &AppState not AppState
    if let Some(handle) = app_state
        .heartbeat
        .lock()
        .await
        .take()
    {
        handle.abort();
    }
}
