use sqlx::{Pool, Postgres, Transaction};
use uuid::Uuid;

use crate::{data_logic::{connection_data::{d_add_connection, d_get_user_role, d_get_user_sessions, d_remove_all_connections_to_session, d_remove_connection}, session_data::{d_add_session, d_get_session_list, d_remove_session}}, errors::api_error::ApiError, logic::auth_logic::l_check_active_session, modules::{active_sessions::SessionMap, connection::Connection, session::{Session, SessionRole}}};

pub async fn l_get_session_list(db_pool: Pool<Postgres>, user_key: &String) -> Result<Vec<SessionRole>, ApiError> {
    let user_sessions = match d_get_user_sessions(db_pool.clone(), &user_key).await {
        Ok(c) => c,
        Err(_e) => return Err(ApiError::NotFound)
    };
    Ok(user_sessions)
}

pub async fn l_create_session(db_pool: Pool<Postgres>, user_key: &String, session_name: &String) -> Result<String, ApiError> {
    let session_list = match d_get_session_list(db_pool.clone()).await {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    let new_session: Session = Session { session_key: l_generate_session_key(&session_list), 
                                         session_name: session_name.clone() };
    let session_key = new_session.session_key.clone();

    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match d_add_session(&mut tx, &new_session).await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    
    let new_connection: Connection = Connection { user_key: user_key.clone(), session_key: session_key.clone(), user_role: "owner".into() };
    match d_add_connection(&mut tx, &new_connection).await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match tx.commit().await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    Ok(session_key.clone())
}

pub async fn l_add_session(db_pool: Pool<Postgres>, user_key: &String, session_key: &String) -> Result<(), ApiError> {
    
    let new_connection: Connection = Connection { user_key: user_key.clone(), session_key: session_key.clone(), user_role: "member".into() };
    
    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    
    match d_add_connection(&mut tx, &new_connection).await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    // db handle all errors

    match tx.commit().await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    Ok(())
}

pub async fn l_forget_session(db_pool: Pool<Postgres>, user_key: &String, session_key: &String) -> Result<(), ApiError> {
    
    let connection = match d_get_user_role(db_pool.clone(), &user_key, &session_key).await {
        Ok(c) => c,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    if connection.user_role != "member" {
        return Err(ApiError::InvalidInput("User is owner of session. He can delete session, but not forget".into()));
    }

    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match d_remove_connection(&mut tx, &user_key, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match tx.commit().await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    Ok(())
}

pub async fn l_forget_session_by_tx(db_pool: Pool<Postgres>, mut tx: &mut Transaction<'_, Postgres>, user_key: &String, session_key: &String) -> Result<(), ApiError> {
    
    let connection = match d_get_user_role(db_pool.clone(), &user_key, &session_key).await {
        Ok(c) => c,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    if connection.user_role != "member" {
        return Err(ApiError::InvalidInput("User is owner of session. He can delete session, but not forget".into()));
    }

    match d_remove_connection(&mut tx, &user_key, &session_key).await {
        Ok(()) => Ok(()),
        Err(e) => Err(ApiError::InvalidInput(e.to_string()))
    }
}

pub async fn l_delete_session_by_owner(db_pool: Pool<Postgres>, active_sessions: SessionMap, user_key: &String, session_key: &String) -> Result<(), ApiError> {
    
    match l_check_active_session(active_sessions, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    let connection = match d_get_user_role(db_pool.clone(), &user_key, &session_key).await {
        Ok(c) => c,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    if connection.user_role != "owner" {
        return Err(ApiError::InvalidInput("User is not owner of session. He can forget session, but not delete".into()));
    }

    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match d_remove_all_connections_to_session(&mut tx, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match d_remove_session(&mut tx, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match tx.commit().await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    Ok(())
}

pub async fn l_delete_session_by_owner_by_tx(db_pool: Pool<Postgres>, active_sessions: SessionMap, mut tx: &mut Transaction<'_, Postgres>, user_key: &String, session_key: &String) -> Result<(), ApiError> {
    
    match l_check_active_session(active_sessions, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }

    let connection = match d_get_user_role(db_pool.clone(), &user_key, &session_key).await {
        Ok(c) => c,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    if connection.user_role != "owner" {
        return Err(ApiError::InvalidInput("User is not owner of session. He can forget session, but not delete".into()));
    }

    match d_remove_all_connections_to_session(&mut tx, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match d_remove_session(&mut tx, &session_key).await {
        Ok(()) => Ok(()),
        Err(e) => Err(ApiError::InvalidInput(e.to_string()))
    }
}

// TODO: check it for security. for now it should be ok, but it is not ideal.
fn l_generate_session_key(session_list: &Vec<Session>) -> String {
    let mut s_key: String;
    loop {
        s_key = Uuid::new_v4().to_string();
        match session_list.iter().find(|s| s.session_key == s_key) {
            None => break,
            Some(_) => continue
        } 
    };

    return s_key;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::session::Session;

    #[test]
    fn test_generate_session_key_is_not_empty() {
        let sessions: Vec<Session> = vec![];
        let key = l_generate_session_key(&sessions);
        assert!(!key.is_empty());
    }

    #[test]
    fn test_generate_session_key_is_valid_uuid() {
        use uuid::Uuid;
        let sessions: Vec<Session> = vec![];
        let key = l_generate_session_key(&sessions);
        assert!(Uuid::parse_str(&key).is_ok(), "not a valid uuid: {}", key);
    }

    #[test]
    fn test_generate_session_key_is_36_chars() {
        let sessions: Vec<Session> = vec![];
        let key = l_generate_session_key(&sessions);
        assert_eq!(key.len(), 36);
    }
}