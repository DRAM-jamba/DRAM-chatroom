use crate::{data_logic::user_data::{add_user, get_user_list}, logic::auth_logic::generate_auth_token, modules::{api_error::ApiError, app_error::AppError, user::User}};

pub fn add_user_to_server() -> Result<(String, String), ApiError> {
    let user_list = match get_user_list() {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InternalError) // TODO: change it later
    };

    let new_id = match user_list.last() {
        None => 0,
        Some(u) => u.id + 1
    };

    let new_user: User = User {id: new_id, user_key: generate_user_key(), 
                               related_session_keys: [].to_vec(), 
                               last_time_seen: chrono::Utc::now().timestamp() };
    match add_user(&new_user) {
        Ok(()) => (),
        Err(e) => {
            return Err(ApiError::InternalError) // TODO: change it later
        }
    }

    Ok((generate_auth_token(), new_user.user_key))
}


pub fn connect_user_to_server(user_key: String) -> Result<String, ApiError> {
    let user_list = get_user_list();

    match user_list {
        Ok(user_list) => {
            match user_list.iter().find(|u| u.user_key == user_key).cloned() {
                None => Err(ApiError::NotFound),
                Some(u) => Ok(generate_auth_token())
            }
        }
        Err(e) => {
            match e {
                // TODO: change it later, but how?
                AppError::Io(msg) => Err(ApiError::InvalidInput(msg.to_string())),
                AppError::Json(msg) => Err(ApiError::InvalidInput(msg.to_string()))
            }
        }
    }
}

fn generate_user_key() -> String {
    // TODO: implement this function

    return "hhee22HAM4433".into();
}