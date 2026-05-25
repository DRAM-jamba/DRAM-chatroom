use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{data_logic::{connection_data::d_get_user_connections, user_data::{d_add_user, d_get_user, d_remove_user, d_update_user}}, 
                        errors::api_error::ApiError, 
                        logic::{auth_logic::l_generate_auth_token, session_logic::{l_delete_session_by_owner_by_tx, l_forget_session_by_tx}}, 
                        modules::{active_sessions::SessionMap, user::User}};

pub async fn l_add_user_to_server(db_pool: Pool<Postgres>) -> Result<String, ApiError> {

    let new_user: User = User {user_key: l_generate_user_key(), 
                               nickname: chrono::Utc::now().to_string(), // nickname must be unique, should work 
                               last_time_seen: chrono::Local::now().naive_local() };
    
    match d_add_user(db_pool.clone(), &new_user).await {
        Ok(()) => (),
        Err(e) => return Err(e.into())
    };

    Ok(new_user.user_key)
}

pub async fn l_connect_user_to_server(db_pool: Pool<Postgres>, user_key: String) -> Result<String, ApiError> {
    let _user = match d_get_user(db_pool.clone(), &user_key).await {
        Ok(u) => u,
        Err(e) => return Err(e.into())
    };
    Ok(l_generate_auth_token())
}

pub async fn l_delete_user_from_server(db_pool: Pool<Postgres>, active_sessions: SessionMap, user_key: String) -> Result<(), ApiError> {
    let connections = match d_get_user_connections(db_pool.clone(), &user_key).await {
        Ok(c) => c,
        Err(e) => return Err(e.into())
    };

    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(_e) => return Err(ApiError::InternalError)
    };

    for c in connections.iter() {
        if c.user_role == "member" {
            match l_forget_session_by_tx(&mut tx, &c.user_key, &c.session_key).await {
                Ok(()) => (),
                Err(e) => return Err(e)
            };
        }
        else if c.user_role == "owner" {
                match l_delete_session_by_owner_by_tx(active_sessions.clone(), &mut tx, &c.session_key).await {
                Ok(()) => (),
                Err(e) => return Err(e)
            };
        }
    };

    match d_remove_user(&mut tx, &user_key).await {
        Ok(()) => (),
        Err(e) => return Err(e.into())
    };

    match tx.commit().await {
        Ok(()) => (),
        Err(_e) => return Err(ApiError::InternalError)
    };

    Ok(())
}

pub async fn l_set_user_nickname(db_pool: Pool<Postgres>, user_key: String, nickname: String) -> Result<(), ApiError> {
    let mut user = match d_get_user(db_pool.clone(), &user_key).await {
        Ok(u) => u,
        Err(e) => return Err(e.into())
    };
    user.nickname = nickname;

    match d_update_user(db_pool.clone(), &user).await {
        Ok(()) => (),
        Err(e) => return Err(e.into())
    };

    Ok(())
}

// TODO: check it for security. for now it should be ok, but it is not ideal.
fn l_generate_user_key() -> String {
    let u_key: String = Uuid::new_v4().to_string();

    u_key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::user::User;

    #[test]
    fn test_generate_user_key_is_not_empty() {
        let key = l_generate_user_key();
        assert!(!key.is_empty());
    }

    #[test]
    fn test_generate_user_key_is_valid_uuid() {
        let key = l_generate_user_key();
        let parsed = Uuid::parse_str(&key);
        assert!(parsed.is_ok(), "key was not a valid uuid: {}", key);
    }

    #[test]
    fn test_generate_user_key_is_36_chars() {
        let key = l_generate_user_key();
        assert_eq!(key.len(), 36);
    }
}