use serde::{Deserialize, Serialize};


#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub user_key: String,
    pub nickname: String,
    pub related_session_keys: Vec<String>,
    pub last_time_seen: i64 // timestamp,
}

// TODO: make methods 
