use uuid::Uuid;

use crate::{data_logic::{session_data::get_session_list, user_data::{add_user, get_user_by_user_key, get_user_list, remove_user, update_user}}, 
                        errors::api_error::ApiError, 
                        logic::{auth_logic::generate_auth_token, session_logic::delete_session_by_owner}, 
                        modules::user::User};

pub fn add_user_to_server() -> Result<(String, String), ApiError> {
    let user_list = match get_user_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InternalError) // TODO: change it later
    };

    let new_id = match user_list.last() {
        None => 0,
        Some(u) => u.id + 1
    };

    let new_user: User = User {id: new_id, user_key: generate_user_key(&user_list), nickname: "".into(), 
                               related_session_keys: [].to_vec(), 
                               last_time_seen: chrono::Utc::now().timestamp(), in_session: false };
    match add_user(&new_user) {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InternalError) // TODO: change it later
    };

    Ok((generate_auth_token(), new_user.user_key))
}


pub fn connect_user_to_server(user_key: String) -> Result<String, ApiError> {
    let user = match get_user_by_user_key(user_key) {
        Ok(u) => u,
        Err(e) => return Err(ApiError::NotFound)
    };
    Ok(generate_auth_token())
}

pub fn delete_user_from_server(user_key: String) -> Result<(), ApiError> {
    let user = match get_user_by_user_key(user_key) {
        Ok(u) => u,
        Err(e) => return Err(ApiError::NotFound)
    };
    let session_list = match get_session_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    for s in session_list.iter() {
        if s.session_owner_id == user.id {
            match delete_session_by_owner(&user.user_key, &s.session_key) {
                Ok(()) => (),
                Err(e) => return Err(e)
            }
        }
    };

    match remove_user(&user) {
        Ok(()) => Ok(()),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    }

}

pub fn set_user_nickname(user_key: String, nickname: String) -> Result<(), ApiError> {
    let mut user = match get_user_by_user_key(user_key) {
        Ok(u) => u,
        Err(e) => return Err(ApiError::NotFound)
    };
    user.nickname = nickname;
    match update_user(&user) {
        Ok(()) => Ok(()),
        Err(e) => Err(ApiError::InvalidInput(e.to_string()))
    }
}

// TODO: check it for security. for now it should be ok, but it is not ideal.
fn generate_user_key(user_list: &Vec<User>) -> String {
    let mut u_key: String;
    loop {
        u_key = Uuid::new_v4().to_string();
        match user_list.iter().find(|u| u.user_key == u_key) {
            None => break,
            Some(u) => continue
        } 
    };

    return u_key;
}