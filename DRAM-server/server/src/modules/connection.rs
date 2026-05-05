use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;


#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct Connection {
    pub user_key: String,
    pub session_key: String,
    pub user_role: String
}
