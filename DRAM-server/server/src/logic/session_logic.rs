use crate::{data_logic::{session_data::{add_session, get_session_list}, user_data::{add_user, get_user_list, update_user}}, errors::api_error::ApiError, modules::{session::Session, user}};

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

pub fn create_session_l(user_key: String, session_name: String) -> Result<(), ApiError> {
    let user_list = match get_user_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    let user = match user_list.iter().find(|u| u.user_key == user_key) {
        None => return Err(ApiError::NotFound),
        Some(u) => u
    };
    let session_list = match get_session_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    let new_id = match session_list.last() {
        None => 0,
        Some(i) => i.session_id + 1
    };


    let new_session: Session = Session { session_id: new_id, session_key: generate_session_key(), 
                                         session_owner_id: user.id, 
                                         name: session_name, chat_log: [].to_vec(), 
                                         current_user_list: [].to_vec(), black_list: [].to_vec() };
    match add_session(&new_session) {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    match add_session_by_session_key(user_key, new_session.session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

pub fn add_session_by_session_key(user_key: String, session_key: String) -> Result<(), ApiError> {
    let user_list = match get_user_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    let mut user = match user_list.iter().find(|u| u.user_key == user_key).cloned() {
        None => return Err(ApiError::NotFound),
        Some(u) => u
    };
    let session_list = match get_session_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    match session_list.iter().find(|s| s.session_key == session_key) {
        None => return Err(ApiError::NotFound),
        Some(s) => {
            match user.related_session_keys.iter().find(|s| **s == session_key) { // TODO: find how works '*'
                None => {
                    user.related_session_keys.push(session_key);
                    match update_user(&user) {
                        Ok(()) => Ok(()),
                        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
                    }
                },
                Some(s) => Err(ApiError::InvalidInput("User already has this session".into()))
            }
            
        }
    }

}

fn generate_session_key() -> String {
    let mut key = chrono::Utc::now().timestamp().to_string();
    key = key + "sysING";
    return key;
}