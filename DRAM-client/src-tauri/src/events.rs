use serde::Serialize;
use tauri::AppHandle;

#[derive(Clone, Serialize)]
pub struct MessagePayload {
    // The message content, will probably change
    pub from: String,
    pub body: String,
    pub ts: u64,
}

#[derive(Clone, Serialize)]
pub struct SessionPlayload {
    // The session info, will probably change
    pub session_id: String,
    pub users: Vec<String>,
}

pub fn emit_message(app: &AppHandle, raw: &str) {
    let _ = app.emit("message", raw);
}

pub fn emit_session_update(app: &AppHandle, session: &SessionPlayload) {
    let _ = app.emit("session_update", session);
}

pub fn emit_disconnected(app: &AppHandle) {
    let _ = app.emit("disconnected", ());
}