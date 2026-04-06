use std::{fs::{self, read_to_string}, path::Path};

use serde_json::{from_str, to_string_pretty};

use crate::{errors::app_error::AppError, modules::session::Session};


const SESSION_LIST_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/session_list.json");

pub fn get_session_list() -> Result<Vec<Session>, AppError> {
    let json_path: &Path = Path::new(SESSION_LIST_PATH);
    
    let data = match read_to_string(json_path) {
        Ok(s) if s.trim().is_empty() => String::from("[]"),
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::from("[]"),
        Err(e) => return Err(e.into()),
    };

    let vec: Vec<Session> = match from_str(&data) {
        Ok(v) => v,
        Err(e) => return Err(e.into())
    };

    Ok(vec)
}

// now this funciton delete session_list for moment, so here may be mistakes.
// TODO: change later to DB or file blocking. security
pub fn save_session_list(user_list: Vec<Session>) -> Result<(), AppError> {
    let json_path: &Path = Path::new(SESSION_LIST_PATH);

    let new_json: String = match to_string_pretty(&user_list) {
        Ok(v) => v,
        Err(e) => return Err(e.into())
    };

    let tmp_path = json_path.with_extension("json.tmp");

    match fs::write(&tmp_path, new_json) {
        Ok(()) => (),
        Err(e) => return Err(e.into())
    };
    match fs::rename(&tmp_path, &json_path) {
        Ok(()) => (),
        Err(e) => return Err(e.into())
    };

    Ok(())
}

pub fn get_session_by_session_key(session_key: String) -> Result<Session, AppError> {
    let session_list = match get_session_list() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };
    let session = match session_list.iter().find(|s| s.session_key == session_key) {
        None => return Err(AppError::Else("Session not found".into())),
        Some(s) => s
    };
    Ok(session.clone())
}

pub fn add_session(session: &Session) -> Result<(), AppError> {
    let mut vec = match get_session_list() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    vec.push(session.clone());

    match save_session_list(vec) {
        Ok(()) => Ok(()),
        Err(e) => return Err(e)
    }
}

pub fn remove_session(session: &Session) -> Result<(), AppError> {
    let mut vec = match get_session_list() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let s_i = match vec.iter().position(|s| s.session_id == session.session_id) {
        None => return Err(AppError::Else("Session not found in session list".into())),
        Some(i) => i
    };

    vec.remove(s_i);

    match save_session_list(vec) {
        Ok(()) => Ok(()),
        Err(e) => return Err(e)
    }
}