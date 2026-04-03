use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub from: String,
    pub body: String,
    pub ts: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionPayload {
    pub session_id: String,
    pub participants: Vec<String>,
    pub chat_log:     Vec<MessagePayload>,
}

pub fn emit_message(app: &AppHandle, raw: &str) {
    let _ = app.emit("message", raw);
}

pub fn emit_joined_session(app: &AppHandle) {
    let _ = app.emit("joined_session", ());
}

pub fn emit_session_update(app: &AppHandle, payload: SessionPayload) {
    let _ = app.emit("session_update", payload);
}

pub fn emit_disconnected(app: &AppHandle) {
    let _ = app.emit("disconnected", ());
}

pub fn emit_connected(app: &AppHandle) {
    let _ = app.emit("connected", ());
}
