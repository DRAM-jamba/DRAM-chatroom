use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;


#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_key: String,
    pub nickname: String,
    pub last_time_seen: chrono::NaiveDateTime,
    pub public_key: String
}
