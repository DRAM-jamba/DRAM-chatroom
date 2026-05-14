use crate::models::{MemberListPayload, MessagePayload, SessionPayload};
use tauri::{AppHandle, Emitter};

pub fn emit_message(app: &AppHandle, payload: MessagePayload) {
    let _ = app.emit("message", payload);
}

pub fn emit_member_list(app: &AppHandle, participants: Vec<String>) {
    let _ = app.emit("member_list", MemberListPayload { participants });
}

pub fn emit_session_update(app: &AppHandle, payload: SessionPayload) {
    let _ = app.emit("session_update", payload);
}

pub fn emit_disconnected(app: &AppHandle) {
    let _ = app.emit("disconnected", ());
}
