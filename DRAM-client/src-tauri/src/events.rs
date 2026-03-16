use serde::Serialize;
use tauri::{AppHandle, Emitter}; // ← Emitter trait must be in scope in v2

#[derive(Clone, Serialize)]
pub struct MessagePayload {
    pub from: String,
    pub body: String,
    pub ts: u64,
}

#[derive(Clone, Serialize)]
pub struct SessionPayload {
    pub session_id: String,
    pub participants: Vec<String>,
}

pub fn emit_message(app: &AppHandle, raw: &str) {
    let _ = app.emit("message", raw);
}

pub fn emit_session_update(app: &AppHandle, payload: SessionPayload) {
    let _ = app.emit("session_update", payload);
}

pub fn emit_disconnected(app: &AppHandle) {
    let _ = app.emit("disconnected", ());
}
