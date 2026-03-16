use std::time::Duration;
use tokio::time::interval;
use crate::{state::AppState, state::ConnectionState};

pub async fn start(app_state: AppState) {
    let connection = state.connection.clone();
    let heartbeat = state.heartbeat.clone();

    let handle = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            let conn = connection.lock().await;
            if let ConnectionState::Connected(client) = &*conn {
                if client.ping().await.is_err() {
                    break; // server is unreachable, read loop will handle disconnection event
                }
            } else {
                break; // not connected, stop heartbeat
            }
        }
    });

    *heartbeat.lock().await = Some(handle);
}

pub async fn stop(app_state: AppState) {
    if let Some(Handle) = heartbeat.lock().await.take() {
        Handle.abort();
    }
}