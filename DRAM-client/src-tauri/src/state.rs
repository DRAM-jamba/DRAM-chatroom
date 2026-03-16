use std::sync::Arc;
use tokio::sync::Mutex;
use crate::websocket::WsClient;

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
    Recconnecting {
        attempts: u32,
    },
}

pub struct AppState {
    pub connection: Arc<Mutex<ConnectionState>>,
    pub session: Arc<Mutex<SessionState>>,
    pub heartbeat: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            session: Arc::new(Mutex::new(SessionState::Idle)),
            heartbeat: Arc::new(Mutex::new(None)),
        }
    }
}