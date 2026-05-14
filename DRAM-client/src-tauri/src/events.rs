use tauri::{AppHandle, Emitter};
use crate::models::MessageObj;

pub fn emit_message(app: &AppHandle, payload: MessageObj) {
    let _ = app.emit("message", payload);
}

pub fn emit_session_update(app: &AppHandle, payload: MessageObj) {
    let _ = app.emit("session_update", payload);
}

pub fn emit_disconnected(app: &AppHandle) {
    let _ = app.emit("disconnected", ());
}