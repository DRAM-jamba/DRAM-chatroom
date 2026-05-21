use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    // M
    Message,
    Connect,
    Disconnect,
    // CM
    UserList,
    VoiceList,
    VoiceStart,
    VoiceEnd
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageObj {
    pub m_type: MessageType,
    pub from: String,
    pub body: String,
    pub ts: i64
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BackMessageObj {
    pub m_type: MessageType,
    pub body: String
}