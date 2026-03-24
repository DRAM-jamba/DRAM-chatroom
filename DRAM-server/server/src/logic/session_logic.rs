use crate::{data_logic::{session_data::get_session_list, user_data::get_user_list}, errors::api_error::ApiError, modules::{session::Session, user}};

pub fn get_user_related_session_list(user_key: String) -> Result<Vec<Session>, ApiError> {
    let user_list = match get_user_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string())) // TODO: change it later
    };
    let user = match user_list.iter().find(|u| u.user_key == user_key) {
        None => return Err(ApiError::NotFound),
        Some(u) => u
    };
    let session_list = match get_session_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    let users_sessions: Vec<Session> = session_list.into_iter()
                                                   .filter(|s| user.related_session_keys.contains(&s.session_key))
                                                   .collect();
    Ok(users_sessions)
}