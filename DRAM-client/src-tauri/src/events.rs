use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Message,
    Connect,
    Disconnect,
    UserList,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageObj {
    pub m_type: MessageType,
    pub from: String,
    pub body: String,
    pub ts: i64,
}

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
    pub chat_log: Vec<MessagePayload>,
}

// Helper for simple member updates
#[derive(Clone, Serialize, Deserialize)]
pub struct MemberListPayload {
    pub participants: Vec<String>,
}

pub fn emit_message(app: &AppHandle, payload: MessagePayload) {
    let _ = app.emit("message", payload);
}

// This matches what subscribeToMembers in chatService.ts expects
pub fn emit_member_list(app: &AppHandle, participants: Vec<String>) {
    let _ = app.emit("member_list", MemberListPayload { participants });
}

pub fn emit_session_update(app: &AppHandle, payload: SessionPayload) {
    let _ = app.emit("session_update", payload);
}

pub fn emit_disconnected(app: &AppHandle) {
    let _ = app.emit("disconnected", ());
}