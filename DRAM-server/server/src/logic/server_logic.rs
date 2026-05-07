use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{data_logic::{connection_data::d_get_user_connections, user_data::{d_add_user, d_get_user, d_get_user_list, d_remove_user, d_update_user}}, 
                        errors::api_error::ApiError, 
                        logic::{auth_logic::l_generate_auth_token, session_logic::{l_delete_session_by_owner_by_tx, l_forget_session_by_tx}}, 
                        modules::{active_sessions::SessionMap, user::User}};

pub async fn l_add_user_to_server(db_pool: Pool<Postgres>) -> Result<String, ApiError> {
    let user_list = match d_get_user_list(db_pool.clone()).await {
        Ok(v) => v,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string())) // TODO: change it later
    };

    let new_user: User = User {user_key: l_generate_user_key(&user_list), nickname: "".into(), 
                               last_time_seen: chrono::Local::now().naive_local() };
    
    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    
    match d_add_user(&mut tx, &new_user).await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string())) // TODO: change it later
    };

    match tx.commit().await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    Ok(new_user.user_key)
}


pub async fn l_connect_user_to_server(db_pool: Pool<Postgres>, user_key: String) -> Result<String, ApiError> {
    let _user = match d_get_user(db_pool.clone(), &user_key).await {
        Ok(u) => u,
        Err(_e) => return Err(ApiError::NotFound)
    };
    Ok(l_generate_auth_token())
}

pub async fn l_delete_user_from_server(db_pool: Pool<Postgres>, active_sessions: SessionMap, user_key: String) -> Result<(), ApiError> {
    let connections = match d_get_user_connections(db_pool.clone(), &user_key).await {
        Ok(c) => c,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    for c in connections.iter() {
        if c.user_role == "member" {
            match l_forget_session_by_tx(db_pool.clone(), &mut tx, &c.user_key, &c.session_key).await {
                Ok(()) => (),
                Err(e) => return Err(e)
            };
        }
        else if c.user_role == "owner" {
                match l_delete_session_by_owner_by_tx(db_pool.clone(), active_sessions.clone(), &mut tx, &c.user_key, &c.session_key).await {
                Ok(()) => (),
                Err(e) => return Err(e)
            };
        }
    };

    match d_remove_user(&mut tx, &user_key).await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match tx.commit().await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    Ok(())

}

pub async fn l_set_user_nickname(db_pool: Pool<Postgres>, user_key: String, nickname: String) -> Result<(), ApiError> {
    let mut user = match d_get_user(db_pool.clone(), &user_key).await {
        Ok(u) => u,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    user.nickname = nickname;

    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match d_update_user(&mut tx, &user).await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    match tx.commit().await {
        Ok(()) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    Ok(())
}

// TODO: check it for security. for now it should be ok, but it is not ideal.
fn l_generate_user_key(user_list: &Vec<User>) -> String {
    let mut u_key: String;
    loop {
        u_key = Uuid::new_v4().to_string();
        match user_list.iter().find(|u| u.user_key == u_key) {
            None => break,
            Some(_) => continue
        } 
    };

    return u_key;
}