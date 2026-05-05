use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageObj {
    pub from: String,
    pub body: String,
    pub ts: i64
}