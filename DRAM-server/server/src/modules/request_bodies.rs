use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserKey {
    pub user_key: String
}

#[derive(Deserialize)]
pub struct UserKeyNickname {
    pub user_key: String,
    pub nickname: String
}

#[derive(Deserialize)]
pub struct UserSessionKeys {
    pub user_key: String,
    pub session_key: String
}

#[derive(Deserialize)]
pub struct UserKeySessionName {
    pub user_key: String,
    pub session_name: String
}

#[derive(Deserialize)]
pub struct PublicKey {
    pub public_key: String
}

#[derive(Deserialize)]
pub struct ChallengeResponse {
    pub nonce: String,
    pub signature: String,
    pub user_key: String
}