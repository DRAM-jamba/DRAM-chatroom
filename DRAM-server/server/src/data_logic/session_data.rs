use sqlx::{Pool, Postgres, Transaction};

use crate::{errors::app_error::AppError, modules::session::Session};


pub async fn d_get_session_list(db_pool: Pool<Postgres>) -> Result<Vec<Session>, AppError> {

    sqlx::query_as::<_, Session>("SELECT session_key, session_name FROM sessions")
                                .fetch_all(&db_pool).await
                                .map_err(|e| AppError::Database(e))
}

pub async fn d_get_session(db_pool: Pool<Postgres>, session_key: &String) -> Result<Session, AppError> {

    sqlx::query_as::<_, Session>("SELECT session_key, session_name FROM sessions WHERE session_key = $1")
                    .bind(&session_key)
                    .fetch_one(&db_pool).await
                    .map_err(|e| AppError::Database(e))
}

pub async fn d_create_session(tx: &mut Transaction<'_, Postgres>, session: &Session) -> Result<(), AppError> {

    let _result = sqlx::query("INSERT INTO sessions (session_key, session_name) VALUES ($1, $2)")
                    .bind(&session.session_key)
                    .bind(&session.session_name)
                    .execute(&mut **tx).await?; 
    // as i understood, if will be error, it will automatically change itself to 
    // AppError::Database, so it should be fine

    Ok(())
}

pub async fn d_delete_session(tx: &mut Transaction<'_, Postgres>, session_key: &String) -> Result<(), AppError> {

    let result = sqlx::query("DELETE FROM sessions WHERE session_key = $1")
                    .bind(&session_key)
                    .execute(&mut **tx).await?; 
    // as i understood, if will be error, it will automatically change itself to 
    // AppError::Database, so it should be fine

    if result.rows_affected() == 0 {
        return Err(AppError::Database(sqlx::Error::RowNotFound));
    }

    Ok(())
}