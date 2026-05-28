use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use sqlx::{Pool, Postgres};
use tokio::sync::RwLock;

use crate::modules::{active_nonce::{NonceMap}, active_sessions::SessionMap, active_users::UsersMap};


pub type ServerState = Arc<ServerInfo>;

#[derive(Clone)]
pub struct ServerInfo {
    pub db_pool: Pool<Postgres>,
    pub active_sessions: SessionMap,
    pub active_users: UsersMap,
    pub active_nonce: NonceMap
}

impl ServerInfo {
    pub fn new(db_pool: Pool<Postgres>) -> Self {
        let db_pool = db_pool;
        let active_sessions: SessionMap = Arc::new(RwLock::new(HashMap::new()));
        let active_users: UsersMap = Arc::new(RwLock::new(HashMap::new()));
        let active_nonce: NonceMap = Arc::new(DashMap::new());
        Self {
            db_pool,
            active_sessions,
            active_users,
            active_nonce
        }
    }
}