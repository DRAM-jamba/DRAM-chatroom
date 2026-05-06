use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Message,
    Connect,
    Disconnect,
    UserList
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageObj {
    pub m_type: MessageType,
    pub from: String,
    pub body: String,
    pub ts: i64
}