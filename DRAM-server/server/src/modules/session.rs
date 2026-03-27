use serde::{Deserialize, Serialize};


#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: u64,
    pub session_key: String,
    pub session_owner_id: u64,
    pub name: String,
    pub chat_log: Vec<String>,
    pub current_user_list: Vec<u64>,
    pub black_list: Vec<u64>,
}