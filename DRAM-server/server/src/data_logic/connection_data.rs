use sqlx::{Pool, Postgres, Transaction};

use crate::{errors::app_error::AppError, modules::{connection::Connection, session::SessionRole}};



pub async fn d_get_user_sessions(db_pool: Pool<Postgres>, user_key: &String) -> Result<Vec<SessionRole>, AppError> {
  
    sqlx::query_as::<_, SessionRole>("SELECT sessions.session_key, session_name, user_role 
                                                      FROM user_session
                                                      JOIN sessions
                                                      ON sessions.session_key = user_session.session_key 
                                                      WHERE user_key = $1")
                                .bind(&user_key)
                                .fetch_all(&db_pool).await
                                .map_err(|e| AppError::Database(e))
}

pub async fn d_get_user_connections(db_pool: Pool<Postgres>, user_key: &String) -> Result<Vec<Connection>, AppError> {
  
    sqlx::query_as::<_, Connection>("SELECT user_key, session_key, user_role FROM user_session WHERE user_key = $1")
                                .bind(&user_key)
                                .fetch_all(&db_pool).await
                                .map_err(|e| AppError::Database(e))
}

pub async fn d_get_user_role(db_pool: Pool<Postgres>, user_key: &String, session_key: &String) -> Result<Connection, AppError> {
  
    sqlx::query_as::<_, Connection>("SELECT user_key, session_key, user_role FROM user_session WHERE user_key = $1 AND session_key = $2")
                                .bind(&user_key)
                                .bind(&session_key)
                                .fetch_one(&db_pool).await
                                .map_err(|e| AppError::Database(e))
}

pub async fn d_add_connection(tx: &mut Transaction<'_, Postgres>, connection: &Connection) -> Result<(), AppError> {
  
    let _result = sqlx::query("INSERT INTO user_session (user_key, session_key, user_role) VALUES ($1, $2, $3)")
                                .bind(&connection.user_key)
                                .bind(&connection.session_key)
                                .bind(&connection.user_role)
                                .execute(&mut **tx).await?;
    // as i understood, if will be error, it will automatically change itself to 
    // AppError::Database, so it should be fine

    Ok(())
}

pub async fn d_remove_connection(tx: &mut Transaction<'_, Postgres>, user_key: &String, session_key: &String) -> Result<(), AppError> {
  
    let result = sqlx::query("DELETE FROM user_session WHERE user_key = $1 AND session_key = $2")
                                .bind(&user_key)
                                .bind(&session_key)
                                .execute(&mut **tx).await?;
    // as i understood, if will be error, it will automatically change itself to 
    // AppError::Database, so it should be fine

    if result.rows_affected() == 0 {
        return Err(AppError::Database(sqlx::Error::RowNotFound));
    }

    Ok(())
}

pub async fn d_remove_all_connections_to_session(tx: &mut Transaction<'_, Postgres>, session_key: &String) -> Result<(), AppError> {
  
    let result = sqlx::query("DELETE FROM user_session WHERE session_key = $1")
                                .bind(&session_key)
                                .execute(&mut **tx).await?;
    // as i understood, if will be error, it will automatically change itself to 
    // AppError::Database, so it should be fine

    if result.rows_affected() == 0 {
        return Err(AppError::Database(sqlx::Error::RowNotFound));
    }

    Ok(())
}