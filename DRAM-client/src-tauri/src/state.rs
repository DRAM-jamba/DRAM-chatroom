use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistedServer {
    pub ip: String,
    pub nickname: String,
    pub user_key: String,
}

#[derive(Debug)]
pub enum ConnectionState {
    Disconnected,
    JoinedServer,
    Connected,
}

#[derive(Debug, Clone)]
pub enum SessionState {
    Idle,
    JoinedSession,
    Reconnecting,
}

pub struct AppState {
    pub servers: Arc<Mutex<Vec<PersistedServer>>>,
    pub current_ip: Arc<Mutex<Option<String>>>,
    pub connection: Arc<Mutex<ConnectionState>>,
    pub session: Arc<Mutex<SessionState>>,
    pub heartbeat: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    app_handle: AppHandle,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            servers: Arc::new(Mutex::new(Vec::new())),
            current_ip: Arc::new(Mutex::new(None)),
            connection: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            session: Arc::new(Mutex::new(SessionState::Idle)),
            heartbeat: Arc::new(Mutex::new(None)),
            app_handle,
        }
    }

    pub async fn load_persisted_data(&self) {
        if let Ok(store) = self.app_handle.store("servers.json") {
            if let Some(servers) = store.get("servers") {
                if let Ok(servers_vec) = serde_json::from_value::<Vec<PersistedServer>>(servers) {
                    *self.servers.lock().await = servers_vec;
                }
            }
        }
    }

    pub async fn save_servers(&self) -> Result<(), Box<dyn std::error::Error>> {
        let servers = self.servers.lock().await.clone();
        let store = self.app_handle.store("servers.json")?;
        store.set("servers", serde_json::to_value(&servers)?);
        store.save()?;
        Ok(())
    }

    pub async fn add_server(&self, ip: String, nickname: String, user_key: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut servers = self.servers.lock().await;
        if servers.iter().any(|s| s.ip == ip) {
            return Err("Server already exists".into());
        }
        servers.push(PersistedServer { ip, nickname, user_key });
        drop(servers);
        self.save_servers().await?;
        Ok(())
    }

    pub async fn remove_server(&self, ip: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut servers = self.servers.lock().await;
        servers.retain(|s| s.ip != ip);
        drop(servers);
        self.save_servers().await?;
        Ok(())
    }

    pub async fn get_server(&self, ip: &str) -> Option<PersistedServer> {
        self.servers.lock().await.iter().find(|s| s.ip == ip).cloned()
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