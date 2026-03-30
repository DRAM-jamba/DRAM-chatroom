use crate::websocket::WsClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug)]
pub enum ConnectionState {
    Disconnected,
    JoinedServer {
        ip: String,
    },
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
    /// Maps IP -> user_key for each known server
    pub known_servers: Arc<Mutex<HashMap<String, String>>>,
    /// The IP of the currently active connection
    pub current_ip: Arc<Mutex<Option<String>>>,
    pub client: reqwest::Client,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            session: Arc::new(Mutex::new(SessionState::Idle)),
            heartbeat: Arc::new(Mutex::new(None)),
            known_servers: Arc::new(Mutex::new(HashMap::new())),
            current_ip: Arc::new(Mutex::new(None)),
            client: reqwest::Client::new(),
        }
    }
}
