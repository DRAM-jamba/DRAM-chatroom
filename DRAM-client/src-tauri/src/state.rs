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
}