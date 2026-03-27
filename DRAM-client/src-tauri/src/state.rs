use crate::websocket::WsClient;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum ConnectionState {
    Disconnected,
    Connected(WsClient),
}

#[derive(Debug, Clone)]
pub enum SessionState {
    Idle,
    JoinedSession {
        // If requirements change, we can add more fields here
        session_id: String,
        participants: Vec<String>,
    },
    Reconnecting {
        attempts: u32,
    },
}

pub struct AppState {
    pub connection: Arc<Mutex<ConnectionState>>,
    pub session: Arc<Mutex<SessionState>>,
    pub heartbeat: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    pub current_user_key: Arc<Mutex<Option<String>>>,
    pub current_ip: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            session: Arc::new(Mutex::new(SessionState::Idle)),
            heartbeat: Arc::new(Mutex::new(None)),
            current_user_key: Arc::new(Mutex::new(None)),
            current_ip: String::new(),
        }
    }
}
