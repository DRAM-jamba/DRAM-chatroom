use sqlx::{Pool, Postgres, Transaction};
use uuid::Uuid;

use crate::{data_logic::{connection_data::{d_create_connection, d_create_connection_by_tx, d_delete_all_connections_to_session, d_delete_connection, d_delete_connection_by_tx, d_get_user_role, d_get_user_sessions}, session_data::{d_create_session, d_delete_session}}, errors::api_error::ApiError, logic::auth_logic::l_ensure_session_is_not_active, modules::{active_sessions::SessionMap, connection::Connection, session::{Session, SessionRole}}};

pub async fn l_get_session_list(db_pool: Pool<Postgres>, user_key: &String) -> Result<Vec<SessionRole>, ApiError> {
    let user_sessions = match d_get_user_sessions(db_pool.clone(), &user_key).await {
        Ok(c) => c,
        Err(e) => return Err(e.into())
    };
    Ok(user_sessions)
}

pub async fn l_create_session(db_pool: Pool<Postgres>, user_key: &String, session_name: &String) -> Result<String, ApiError> {

    let new_session: Session = Session { session_key: l_generate_session_key(), 
                                         session_name: session_name.clone() };
    let session_key = new_session.session_key.clone();

    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match d_create_session(&mut tx, &new_session).await {
        Ok(()) => (),
        Err(e) => return Err(e.into())
    };
    
    let new_connection: Connection = Connection { user_key: user_key.clone(), session_key: session_key.clone(), user_role: "owner".into() };
    match d_create_connection_by_tx(&mut tx, &new_connection).await {
        Ok(()) => (),
        Err(e) => return Err(e.into())
    };

    match tx.commit().await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    Ok(session_key.clone())
}

pub async fn l_add_session(db_pool: Pool<Postgres>, user_key: &String, session_key: &String) -> Result<(), ApiError> {
    
    let new_connection: Connection = Connection { user_key: user_key.clone(), session_key: session_key.clone(), user_role: "member".into() };

    // db handle all errors
    match d_create_connection(db_pool.clone(), &new_connection).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.into())
    }
}

pub async fn l_forget_session(db_pool: Pool<Postgres>, user_key: &String, session_key: &String) -> Result<(), ApiError> {
    
    let connection = match d_get_user_role(db_pool.clone(), &user_key, &session_key).await {
        Ok(c) => c,
        Err(e) => return Err(e.into())
    };

    if connection.user_role != "member" {
        return Err(ApiError::InvalidInput("User is owner of session. He can delete session, but not forget".into()));
    }

    match d_delete_connection(db_pool.clone(), &user_key, &session_key).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.into())
    }
}

pub async fn l_forget_session_by_tx(mut tx: &mut Transaction<'_, Postgres>, user_key: &String, session_key: &String) -> Result<(), ApiError> {
    // role check is done in forget server function
    match d_delete_connection_by_tx(&mut tx, &user_key, &session_key).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.into())
    }
}

pub async fn l_delete_session_by_owner(db_pool: Pool<Postgres>, active_sessions: SessionMap, user_key: &String, session_key: &String) -> Result<(), ApiError> {
    
    match l_ensure_session_is_not_active(active_sessions, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    
    let connection = match d_get_user_role(db_pool.clone(), &user_key, &session_key).await {
        Ok(c) => c,
        Err(e) => return Err(e.into())
    };

    if connection.user_role != "owner" {
        return Err(ApiError::InvalidInput("User is not owner of session. He can forget session, but not delete".into()));
    }

    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match d_delete_all_connections_to_session(&mut tx, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(e.into())
    };

    match d_delete_session(&mut tx, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(e.into())
    };

    match tx.commit().await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    Ok(())
}

pub async fn l_delete_session_by_owner_by_tx(active_sessions: SessionMap, mut tx: &mut Transaction<'_, Postgres>, session_key: &String) -> Result<(), ApiError> {
    
    match l_ensure_session_is_not_active(active_sessions, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(e)
    }
    // role check is done in forget server function
    match d_delete_all_connections_to_session(&mut tx, &session_key).await {
        Ok(()) => (),
        Err(e) => return Err(e.into())
    };

    match d_delete_session(&mut tx, &session_key).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.into())
    }
}

// TODO: check it for security. for now it should be ok, but it is not ideal.
fn l_generate_session_key() -> String {
    let s_key: String = Uuid::new_v4().to_string();
    
    s_key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::session::Session;

    #[test]
    fn test_generate_session_key_is_not_empty() {
        let key = l_generate_session_key();
        assert!(!key.is_empty());
    }

    #[test]
    fn test_generate_session_key_is_valid_uuid() {
        use uuid::Uuid;
        let key = l_generate_session_key();
        assert!(Uuid::parse_str(&key).is_ok(), "not a valid uuid: {}", key);
    }

    #[test]
    fn test_generate_session_key_is_36_chars() {
        let key = l_generate_session_key();
        assert_eq!(key.len(), 36);
    }
}