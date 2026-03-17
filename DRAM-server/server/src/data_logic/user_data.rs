use std::{ fs::{self, read_to_string}, path::Path};

use serde_json::{from_str, to_string_pretty};

use crate::modules::{app_error::AppError, user::User};

const USER_LIST_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/user_list.json");

pub fn get_user_list() -> Result<Vec<User>, AppError> {
    let json_path = USER_LIST_PATH;
    let data = fs::read_to_string(json_path);

    match data {
        Ok(data) => {
            let users: Result<Vec<User>, serde_json::Error> = from_str(&data);
            match users {
                Ok(user_list) => {
                    Ok(user_list)
                }
                Err(e) => {
                    Err(AppError::Json(e))
                }
            }
        }
        Err(e ) => {
            Err(AppError::Io(e))
        }
    }    
}

// now this funciton delete user_list for moment, so here may be mistakes.
// TODO: change later to DB or file blocking. security
pub fn add_user(user: &User) -> Result<(), AppError> {
    let json_path: &Path = Path::new(USER_LIST_PATH);
    
    let data = match read_to_string(json_path) {
        Ok(s) if s.trim().is_empty() => String::from("[]"),
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::from("[]"),
        Err(e) => return Err(e.into()),
    };

    let mut vec: Vec<User> = from_str(&data)?;
    vec.push(user.clone());
    let new_json = to_string_pretty(&vec)?;
    
    let tmp_path = json_path.with_extension("json.tmp");
    fs::write(&tmp_path, new_json)?;
    fs::rename(&tmp_path, &json_path)?;

    Ok(())
}