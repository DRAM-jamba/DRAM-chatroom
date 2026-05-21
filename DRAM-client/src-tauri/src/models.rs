use serde::{Deserialize, Serialize};

// Client structs
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    #[serde(alias = "session_key")]
    pub id: String,
    #[serde(alias = "session_name")]
    pub name: String,
    #[serde(alias = "user_role")]
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

#[derive(Deserialize)]
pub struct UserKey {
    pub user_key: String,
}

// Voice-chat structs
#[derive(Deserialize)]
pub struct VoiceToken {
    pub token: String,
}


#[derive(Serialize)]
pub struct VoiceChatInfo {
    pub token: String,
    pub url: String,
}


// Websocket structs
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Message,
    Connect,
    Disconnect,
    // Server
    UserList,
    VoiceList,
    // Client
    VoiceStart,
    VoiceEnd,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageObj {
    pub m_type: MessageType,
    pub from: String,
    pub body: String,
    pub ts: i64,
}

#[derive(Serialize)]
pub struct BackMessageObj {
    pub m_type: MessageType,
    pub body: String,
}
