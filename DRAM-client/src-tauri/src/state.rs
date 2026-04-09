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
    /// Maps IP - user_key for each known server
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

#[cfg(test)]
mod tests {
    use super::*;

    // basic check - when app starts everything should be empty/disconnected
    #[tokio::test]
    async fn test_starts_disconnected() {
        let s = AppState::new();
        let c = s.connection.lock().await;
        assert!(matches!(*c, ConnectionState::Disconnected));
    }

    #[tokio::test]
    async fn test_session_idle_at_start() {
        let s = AppState::new();
        let sess = s.session.lock().await;
        assert!(matches!(*sess, SessionState::Idle));
    }

    // no servers stored when app first loads
    #[tokio::test]
    async fn test_known_servers_empty() {
        let s = AppState::new();
        let m = s.known_servers.lock().await;
        assert!(m.is_empty());
    }

    #[tokio::test]
    async fn test_current_ip_none_at_start() {
        let s = AppState::new();
        let ip = s.current_ip.lock().await;
        assert!(ip.is_none());
    }

    // testing if we can store a user key for an ip
    // this is basically what add() does in lib.rs
    #[tokio::test]
    async fn test_store_server_key() {
        let s = AppState::new();
        s.known_servers.lock().await.insert("127.0.0.1".into(), "abc123".into());
        let k = s.known_servers.lock().await.get("127.0.0.1").cloned();
        assert_eq!(k, Some("abc123".to_string()));
    }

    // looking up an ip that was never added, should return nothing
    #[tokio::test]
    async fn test_unknown_ip_gives_none() {
        let s = AppState::new();
        let k = s.known_servers.lock().await.get("10.0.0.1").cloned();
        assert!(k.is_none());
    }

    #[tokio::test]
    async fn test_set_current_ip() {
        let s = AppState::new();
        *s.current_ip.lock().await = Some("192.168.0.5".into());
        let ip = s.current_ip.lock().await.clone();
        assert_eq!(ip, Some("192.168.0.5".to_string()));
    }

    // simulating what happens when connect() is called
    // connection goes from Disconnected to JoinedServer
    #[tokio::test]
    async fn test_state_changes_to_joined_server() {
        let s = AppState::new();
        *s.connection.lock().await = ConnectionState::JoinedServer {
            ip: "127.0.0.1".into()
        };
        let c = s.connection.lock().await;
        assert!(matches!(*c, ConnectionState::JoinedServer { .. }));
    }

    // disconnect should reset everything back
    #[tokio::test]
    async fn test_disconnect_resets_connection() {
        let s = AppState::new();
        *s.connection.lock().await = ConnectionState::JoinedServer { ip: "127.0.0.1".into() };
        *s.connection.lock().await = ConnectionState::Disconnected;
        let c = s.connection.lock().await;
        assert!(matches!(*c, ConnectionState::Disconnected));
    }

    #[tokio::test]
    async fn test_disconnect_resets_session_too() {
        let s = AppState::new();
        *s.session.lock().await = SessionState::JoinedSession {
            session_id: "s1".into(),
            participants: vec![],
        };
        // simulate what disconnect() does in lib.rs
        *s.session.lock().await = SessionState::Idle;
        let sess = s.session.lock().await;
        assert!(matches!(*sess, SessionState::Idle));
    }

    // joining a session
    #[tokio::test]
    async fn test_join_session_state() {
        let s = AppState::new();
        *s.session.lock().await = SessionState::JoinedSession {
            session_id: "room42".into(),
            participants: vec!["user1".into()],
        };
        let sess = s.session.lock().await;
        assert!(matches!(*sess, SessionState::JoinedSession { .. }));
    }

    // leave session goes back to idle
    #[tokio::test]
    async fn test_leave_session_back_to_idle() {
        let s = AppState::new();
        *s.session.lock().await = SessionState::JoinedSession {
            session_id: "room42".into(),
            participants: vec![],
        };
        *s.session.lock().await = SessionState::Idle;
        let sess = s.session.lock().await;
        assert!(matches!(*sess, SessionState::Idle));
    }

    // storing multiple servers, map should hold all of them
    #[tokio::test]
    async fn test_multiple_servers() {
        let s = AppState::new();
        s.known_servers.lock().await.insert("192.168.1.1".into(), "k1".into());
        s.known_servers.lock().await.insert("192.168.1.2".into(), "k2".into());
        let n = s.known_servers.lock().await.len();
        assert_eq!(n, 2);
    }
}