use uuid::Uuid;

use crate::{data_logic::{session_data::{add_session, get_session_by_session_key, get_session_list, remove_session}, user_data::{add_user, get_user_by_user_key, get_user_list, save_user_list, update_user}}, errors::api_error::ApiError, modules::{session::Session, user}};

pub fn get_user_related_session_list(user_key: String) -> Result<Vec<Session>, ApiError> {
    let user = match get_user_by_user_key(user_key) {
        Ok(u) => u,
        Err(e) => return Err(ApiError::NotFound)
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
    let user = match get_user_by_user_key(user_key) {
        Ok(u) => u,
        Err(e) => return Err(ApiError::NotFound)
    };
    let session_list = match get_session_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    let new_id = match session_list.last() {
        None => 0,
        Some(i) => i.session_id + 1
    };

    let new_session: Session = Session { session_id: new_id, session_key: generate_session_key(&session_list), 
                                         session_owner_id: user.id, 
                                         name: session_name, chat_log: [].to_vec(), 
                                         current_user_list: [].to_vec(), black_list: [].to_vec() };
    match add_session(&new_session) {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    match add_session_by_session_key(user.user_key, new_session.session_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e)
    }
}

pub fn add_session_by_session_key(user_key: String, session_key: String) -> Result<(), ApiError> {
    let mut user = match get_user_by_user_key(user_key) {
        Ok(u) => u,
        Err(e) => return Err(ApiError::NotFound)
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

pub fn forget_session_by_user(user_key: String, session_key: String) -> Result<(), ApiError> {
    let mut user = match get_user_by_user_key(user_key) {
        Ok(u) => u,
        Err(e) => return Err(ApiError::NotFound)
    };
    let session_list = match get_session_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    match session_list.iter().position(|s| s.session_key == session_key) {
        None => return Err(ApiError::NotFound),
        Some(s_i) => {
            match user.related_session_keys.iter().find(|s| **s == session_key) { // TODO: find how works '*'
                None => Err(ApiError::InvalidInput("User does not have this session".into())),
                Some(s) => {
                    if session_list[s_i].session_owner_id == user.id {
                        return Err(ApiError::InvalidInput("User is owner of session. He can delete session, but not forget".into()))
                    }
                    user.related_session_keys.remove(s_i);
                    match update_user(&user) {
                        Ok(()) => Ok(()),
                        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
                    }
                }
            }
            
        }
    }
}

pub fn delete_session_by_owner(user_key: String, session_key: String) -> Result<(), ApiError> {
    let mut user_list = match get_user_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    let session = match get_session_by_session_key(session_key) {
        Ok(s) => s,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match user_list.iter().find(|u| u.id == session.session_owner_id && u.user_key == user_key) {
        None => return Err(ApiError::NotFound),
        Some(u) => ()
    };

    for u in user_list.iter_mut() {
        let s_i = match u.related_session_keys.iter().position(|s_k| **s_k == session.session_key) {
            None => continue,
            Some(i) => i
        };
        u.related_session_keys.remove(s_i);
    };
    match save_user_list(user_list) {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    match remove_session(&session) {
        Ok(()) => Ok(()),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    }
}

// TODO: check it for security. for now it should be ok, but it is not ideal.
fn generate_session_key(session_list: &Vec<Session>) -> String {
    let mut s_key: String;
    loop {
        s_key = Uuid::new_v4().to_string();
        match session_list.iter().find(|s| s.session_key == s_key) {
            None => break,
            Some(s) => continue
        } 
    };

    return s_key;
}