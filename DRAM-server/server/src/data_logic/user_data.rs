use std::{ fs::{self, read_to_string}, ops::Index, path::Path};

use serde_json::{from_str, to_string_pretty};

use crate::{errors::app_error::AppError, modules::user::User};

const USER_LIST_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/user_list.json");

pub fn get_user_list() -> Result<Vec<User>, AppError> {
    let json_path: &Path = Path::new(USER_LIST_PATH);
    
    let data = match read_to_string(json_path) {
        Ok(s) if s.trim().is_empty() => String::from("[]"),
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::from("[]"),
        Err(e) => return Err(e.into()),
    };

    let vec: Vec<User> = match from_str(&data) {
        Ok(v) => v,
        Err(e) => return Err(e.into())
    };

    Ok(vec)
}

// now this funciton delete user_list for moment, so here may be mistakes.
// TODO: change later to DB or file blocking. security
pub fn save_user_list(user_list: Vec<User>) -> Result<(), AppError> {
    let json_path: &Path = Path::new(USER_LIST_PATH);

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

pub fn get_user_by_user_key(user_key: String) -> Result<User, AppError> {
    let user_list = match get_user_list() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };
    let user = match user_list.iter().find(|u| u.user_key == user_key) {
        None => return Err(AppError::Else("User not found".into())),
        Some(u) => u
    };
    Ok(user.clone())
}


pub fn add_user(user: &User) -> Result<(), AppError> {

    let mut vec = match get_user_list() {
        Ok(v) => v.to_vec(),
        Err(e) => return Err(e)
    };
    
    vec.push(user.clone());
    
    match save_user_list(vec) {
        Ok(()) => Ok(()),
        Err(e) => return Err(e)
    }

}

pub fn update_user(user: &User) -> Result<(), AppError> {

    let mut vec = match get_user_list() {
        Ok(v) => v.to_vec(),
        Err(e) => return Err(e.into())
    };
    
    let index= match vec.iter().position(|u| u.id == user.id) {
        None => return Err(AppError::Else("User not found in the list".into())),
        Some(u) => u
    };

    vec[index] = user.clone();
    
    match save_user_list(vec) {
        Ok(()) => Ok(()),
        Err(e) => return Err(e.into())
    }

}