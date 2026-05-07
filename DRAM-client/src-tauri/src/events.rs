use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Message,
    Connect,
    Disconnect,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageObj {
    pub m_type: MessageType,
    pub from: String,
    pub body: String,
    pub ts: i64,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub from: String,
    pub body: String,
    pub ts: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionPayload {
    pub session_id: String,
    pub participants: Vec<String>,
    pub chat_log:     Vec<MessagePayload>,
}

pub fn emit_message(app: &AppHandle, payload: MessagePayload) {
    let _ = app.emit("message", payload);
}

pub fn emit_joined_session(app: &AppHandle) {
    let _ = app.emit("joined_session", ());
}

pub fn emit_member_list(app: &AppHandle, payload: MessageObj) {
    let _ = app.emit("member_list", payload);
}

pub fn emit_member_update_joined(app: &AppHandle, payload: MessageObj) {
    let _ = app.emit("member_update_joined", payload);
}

pub fn emit_member_update_disconnected(app: &AppHandle, payload: MessageObj) {
    let _ = app.emit("member_update_disconnected", payload);
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
