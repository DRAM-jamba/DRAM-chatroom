use sqlx::{Pool, Postgres, Transaction};

use crate::{errors::app_error::AppError, modules::user::User};

pub async fn d_get_user_list(db_pool: Pool<Postgres>) -> Result<Vec<User>, AppError> {
  
    sqlx::query_as::<_, User>("SELECT user_key, nickname, last_time_seen FROM users")
                                .fetch_all(&db_pool).await
                                .map_err(|e| AppError::Database(e))
}

pub async fn d_get_user(db_pool: Pool<Postgres>, user_key: &String) -> Result<User, AppError> {
 
    sqlx::query_as::<_, User>("SELECT user_key, nickname, last_time_seen FROM users WHERE user_key = $1")
                    .bind(&user_key)
                    .fetch_one(&db_pool).await
                    .map_err(|e| AppError::Database(e))
}


pub async fn d_add_user(tx: &mut Transaction<'_, Postgres>, user: &User) -> Result<(), AppError> {

    let _result = sqlx::query("INSERT INTO users (user_key, nickname, last_time_seen) VALUES ($1, $2, $3)")
                    .bind(&user.user_key)
                    .bind(&user.nickname)
                    .bind(&user.last_time_seen)
                    .execute(&mut **tx).await?; 
    // as i understood, if will be error, it will automatically change itself to 
    // AppError::Database, so it should be fine

    Ok(())
}

pub async fn d_update_user(tx: &mut Transaction<'_, Postgres>, user: &User) -> Result<(), AppError> {

    let result = sqlx::query("UPDATE users SET nickname = $1, last_time_seen = $2 WHERE user_key = $3")
                    .bind(&user.nickname)
                    .bind(&user.last_time_seen)
                    .bind(&user.user_key)
                    .execute(&mut **tx).await?; 
    // as i understood, if will be error, it will automatically change itself to 
    // AppError::Database, so it should be fine

    if result.rows_affected() == 0 {
        return Err(AppError::Database(sqlx::Error::RowNotFound));
    }

    Ok(())
}

pub async fn d_remove_user(tx: &mut Transaction<'_, Postgres>, user_key: &String) -> Result<(), AppError> {
    
    let result = sqlx::query("DELETE FROM users WHERE user_key = $1")
                    .bind(&user_key)
                    .execute(&mut **tx).await?; 
    // as i understood, if will be error, it will automatically change itself to 
    // AppError::Database, so it should be fine

    if result.rows_affected() == 0 {
        return Err(AppError::Database(sqlx::Error::RowNotFound));
    }

    Ok(())
}