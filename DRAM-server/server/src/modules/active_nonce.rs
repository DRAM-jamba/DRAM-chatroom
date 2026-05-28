use std::sync::Arc;

use chrono::{DateTime, Utc};
use dashmap::DashMap;

pub type NonceMap = Arc<DashMap<String, Nonce>>;

#[derive(Clone)]
pub struct Nonce{
    pub nonce: String,
    pub expires_at: DateTime<Utc>
}