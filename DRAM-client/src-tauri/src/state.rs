use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::PersistedServer;
use crate::client::WsClient;

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Disconnected,
    JoinedServer,
    Connected,
}

#[derive(Debug, Clone)]
pub enum SessionState {
    Idle,
    JoinedSession(WsClient),
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

    pub async fn add_server(&self, ip: String, server_name: String, user_key: String) -> Result<String, Box<dyn std::error::Error>> {
        let mut servers = self.servers.lock().await;

        let new_id = Uuid::new_v4().to_string();

        if servers.iter().any(|s| s.ip == ip) {
            return Err("Server already exists".into());
        }

        let new_server = PersistedServer { 
            id: new_id.clone(), 
            ip, 
            server_name, 
            user_key, 
            user_nickname: None
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

    pub async fn save_nickname(&self, ip: &str, nickname: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut servers = self.servers.lock().await;
        if let Some(server) = servers.iter_mut().find(|s| s.ip == ip) {
            server.user_nickname = Some(nickname);
            drop(servers);
            self.save_servers().await?;
            Ok(())
        } else {
            Err("Server not found".into())
        }
    }

    pub async fn get_nickname(&self, ip: &str) -> Option<String> {
        self.servers.lock().await.iter().find(|s| s.ip == ip).map(|s| s.user_nickname.clone().unwrap_or_default())
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
