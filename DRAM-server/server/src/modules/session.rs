use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;


#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub session_key: String,
    pub session_name: String
}

#[derive(FromRow, Clone, Serialize, Deserialize)]
pub struct SessionRole {
    pub session_key: String,
    pub session_name: String,
    pub user_role: String
}