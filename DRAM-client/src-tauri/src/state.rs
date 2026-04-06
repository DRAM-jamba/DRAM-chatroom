use tauri::AppHandle;
use std::collections::HashMap;
use tauri_plugin_store::StoreExt;
use serde::{Deserialize, Serialize};

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
    pub servers: Vec<PersistedServer>,
    pub current_ip: Option<String>,
    pub connection: ConnectionState,
    pub session: SessionState,
    app_handle: AppHandle,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        let mut state = Self {
            servers: Vec::new(),
            current_ip: None,
            connection: ConnectionState::Disconnected,
            session: SessionState::Idle,
            app_handle,
        };
        state.load_persisted_data();
        state
    }

    fn load_persisted_data(&mut self) {
        if let Ok(store) = self.app_handle.store("servers.json") {
            if let Some(servers) = store.get("servers") {
                if let Ok(servers_vec) = serde_json::from_value::<Vec<PersistedServer>>(servers) {
                    self.servers = servers_vec;
                }
            }
        }
    }

    pub fn save_servers(&self) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.app_handle.store("servers.json")?;
        store.set("servers", serde_json::to_value(&self.servers)?);
        store.save()?;
        Ok(())
    }

    pub fn add_server(&mut self, ip: String, nickname: String, user_key: String) -> Result<(), Box<dyn std::error::Error>> {
        if self.servers.iter().any(|s| s.ip == ip) {
            return Err("Server already exists".into());
        }
        self.servers.push(PersistedServer { ip, nickname, user_key });
        self.save_servers()?;
        Ok(())
    }

    pub fn remove_server(&mut self, ip: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.servers.retain(|s| s.ip != ip);
        self.save_servers()?;
        Ok(())
    }

    pub fn get_server(&self, ip: &str) -> Option<&PersistedServer> {
        self.servers.iter().find(|s| s.ip == ip)
    }
}
