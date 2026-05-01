use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistedServer {
    pub id: String,
    #[serde(rename = "ipAddress")]
    pub ip: String,
    #[serde(rename = "name")]
    pub nickname: String,
    pub user_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub name: String,
    #[serde(rename = "lastConnected")]
    pub last_connected: String,
}

#[derive(Debug, Clone)]
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

#[derive(Clone)]
pub struct AppState {
    pub servers: Arc<Mutex<Vec<PersistedServer>>>,
    pub current_ip: Arc<Mutex<Option<String>>>,
    pub connection: Arc<Mutex<ConnectionState>>,
    pub session: Arc<Mutex<SessionState>>,
    pub heartbeat: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    app_handle: Option<AppHandle>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            servers: Arc::new(Mutex::new(Vec::new())),
            current_ip: Arc::new(Mutex::new(None)),
            connection: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            session: Arc::new(Mutex::new(SessionState::Idle)),
            heartbeat: Arc::new(Mutex::new(None)),
            app_handle: Some(app_handle),
        }
    }

    pub async fn load_persisted_data(&self) {
        if let Some(ref handle) = self.app_handle {
            if let Ok(store) = handle.store("servers.json") {
                if let Some(servers) = store.get("servers") {
                    if let Ok(servers_vec) = serde_json::from_value::<Vec<PersistedServer>>(servers) {
                        *self.servers.lock().await = servers_vec;
                    }
                }
            }
        }
    }

    pub async fn save_servers(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref handle) = self.app_handle {
            let servers = self.servers.lock().await.clone();
            let store = handle.store("servers.json")?;
            store.set("servers", serde_json::to_value(&servers)?);
            store.save()?;
        }
        Ok(())
    }

    pub async fn add_server(&self, ip: String, nickname: String, user_key: String) -> Result<String, Box<dyn std::error::Error>> {
        let mut servers = self.servers.lock().await;

        let new_id = Uuid::new_v4().to_string();

        if servers.iter().any(|s| s.ip == ip) {
            return Err("Server already exists".into());
        }

        let new_server = PersistedServer { 
            id: new_id.clone(), 
            ip, 
            nickname, 
            user_key 
        };
        servers.push(new_server);
        drop(servers);
        self.save_servers().await?;
        Ok(new_id)
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

    pub async fn get_server_by_id(&self, id: &str) -> Option<PersistedServer> {
        self.servers.lock().await.iter().find(|s| s.id == id).cloned()
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            servers: Arc::new(Mutex::new(Vec::new())),
            current_ip: Arc::new(Mutex::new(None)),
            connection: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            session: Arc::new(Mutex::new(SessionState::Idle)),
            heartbeat: Arc::new(Mutex::new(None)),
            app_handle: None,  // No app handle in tests
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // basic check - when app starts everything should be empty/disconnected
    #[tokio::test]
    async fn test_starts_disconnected() {
        let s = AppState::new_test();
        let c = s.connection.lock().await;
        assert!(matches!(*c, ConnectionState::Disconnected));
    }

    #[tokio::test]
    async fn test_session_idle_at_start() {
        let s = AppState::new_test();
        let sess = s.session.lock().await;
        assert!(matches!(*sess, SessionState::Idle));
    }

    // no servers stored when app first loads
    #[tokio::test]
    async fn test_known_servers_empty() {
        let s = AppState::new_test();
        let m = s.servers.lock().await;
        assert!(m.is_empty());
    }

    #[tokio::test]
    async fn test_current_ip_none_at_start() {
        let s = AppState::new_test();
        let ip = s.current_ip.lock().await;
        assert!(ip.is_none());
    }

    // testing if we can store a user key for an ip
    // this is basically what add() does in lib.rs
    #[tokio::test]
    async fn test_store_server_key() {
    let s = AppState::new_test();
    {
        let mut servers = s.servers.lock().await;
        servers.push(PersistedServer {
            id: "1".to_string(),
            ip: "127.0.0.1".to_string(),
            nickname: "test".to_string(),  // Add a dummy nickname since it's required
            user_key: "abc123".to_string(),
        });
    }  // Lock is dropped here
    let servers = s.servers.lock().await;
    let k = servers.iter().find(|srv| srv.ip == "127.0.0.1").map(|srv| srv.user_key.clone());
    assert_eq!(k, Some("abc123".to_string()));
    }

    // looking up an ip that was never added, should return nothing
    #[tokio::test]
    async fn test_unknown_ip_gives_none() {
        let s = AppState::new_test();
        let servers = s.servers.lock().await;
        let k = servers.iter().find(|srv| srv.ip == "10.0.0.1").map(|srv| srv.user_key.clone());
        assert!(k.is_none());
    }

    #[tokio::test]
    async fn test_set_current_ip() {
        let s = AppState::new_test();
        *s.current_ip.lock().await = Some("192.168.0.5".into());
        let ip = s.current_ip.lock().await.clone();
        assert_eq!(ip, Some("192.168.0.5".to_string()));
    }

    // simulating what happens when connect() is called
    // connection goes from Disconnected to JoinedServer
    #[tokio::test]
    async fn test_state_changes_to_joined_server() {
        let s = AppState::new_test();
        *s.connection.lock().await = ConnectionState::JoinedServer;
        let c = s.connection.lock().await;
        assert!(matches!(*c, ConnectionState::JoinedServer));
    }

    // disconnect should reset everything back
    #[tokio::test]
    async fn test_disconnect_resets_connection() {
        let s = AppState::new_test();
        *s.connection.lock().await = ConnectionState::JoinedServer;
        *s.connection.lock().await = ConnectionState::Disconnected;
        let c = s.connection.lock().await;
        assert!(matches!(*c, ConnectionState::Disconnected));
    }

    #[tokio::test]
    async fn test_disconnect_resets_session_too() {
    let s = AppState::new_test();
    *s.session.lock().await = SessionState::JoinedSession;
    *s.session.lock().await = SessionState::Idle;
    let sess = s.session.lock().await;
    assert!(matches!(*sess, SessionState::Idle));
    }

    // joining a session
    #[tokio::test]
    async fn test_join_session_state() {
        let s = AppState::new_test();
        *s.session.lock().await = SessionState::JoinedSession;
        let sess = s.session.lock().await;
        assert!(matches!(*sess, SessionState::JoinedSession));
    }

    // leave session goes back to idle
    #[tokio::test]
    async fn test_leave_session_back_to_idle() {
    let s = AppState::new_test();
    *s.session.lock().await = SessionState::JoinedSession;
    *s.session.lock().await = SessionState::Idle;
    let sess = s.session.lock().await;
    assert!(matches!(*sess, SessionState::Idle));
    }

    // storing multiple servers, map should hold all of them
    #[tokio::test]
    async fn test_multiple_servers() {
    let s = AppState::new_test();
    {
        let mut servers = s.servers.lock().await;
        servers.push(PersistedServer {
            id: "1".to_string(),
            ip: "192.168.1.1".to_string(),
            nickname: "srv1".to_string(),
            user_key: "k1".to_string(),
        });
        servers.push(PersistedServer {
            id: "2".to_string(),
            ip: "192.168.1.2".to_string(),
            nickname: "srv2".to_string(),
            user_key: "k2".to_string(),
        });
    }
    let servers = s.servers.lock().await;
    assert_eq!(servers.len(), 2);
    let k1 = servers.iter().find(|srv| srv.ip == "192.168.1.1").map(|srv| srv.user_key.clone());
    assert_eq!(k1, Some("k1".to_string()));
    }
}