use std::{collections::{HashMap, VecDeque}, sync::Arc};

use tokio::sync::{RwLock, broadcast::{self, Sender}};


pub type SessionMap = Arc<RwLock<HashMap<String, SessionChat>>>;

#[derive(Clone)]
pub struct SessionChat {
    pub tx: Sender<String>,
    pub history: Arc<RwLock<VecDeque<String>>>
}

impl SessionChat {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            history: Arc::new(RwLock::new(VecDeque::new()))
        }
    }
}