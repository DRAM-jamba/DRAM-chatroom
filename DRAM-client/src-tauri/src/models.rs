use serde::{Deserialize, Serialize};

// Client structs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistedServer {
    pub id: String,
    #[serde(rename = "ipAddress")]
    pub ip: String,
    #[serde(rename = "name")]
    pub server_name: String,
    pub user_key: String,
    pub user_nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub name: String,
    pub user_role: String,
}

#[derive(Deserialize)]
pub struct SessionList {
    pub user_sessions: Vec<Session>,
}

#[derive(Deserialize)]
pub struct SessionKey {
    pub session_key: String,
}

// Websocket structs
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Message,
    Connect,
    Disconnect,
    UserList,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageObj {
    pub m_type: MessageType,
    pub from: String,
    pub body: String,
    pub ts: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub from: String,
    pub body: String,
    pub ts: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionPayload {
    pub session_id: String,
    pub participants: Vec<String>,
    pub chat_log: Vec<MessagePayload>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MemberListPayload {
    pub participants: Vec<String>,
}