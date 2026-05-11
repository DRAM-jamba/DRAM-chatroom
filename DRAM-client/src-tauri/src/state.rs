use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use uuid::Uuid;
use tauri::Manager;

use crate::client::WsClient;
use crate::models::PersistedServer;

const VAULT_PATH: &str = "server_vault";
const CLIENT_NAME: &str = "secure_storage";

#[derive(Debug, Clone)]
pub enum ServerConnectionState {
    Disconnected,
    Connected,
}

#[derive(Debug, Clone)]
pub enum SessionState {
    Idle,
    JoinedSession(WsClient),
}

#[derive(Clone)]
pub struct AppState {
    servers: Arc<Mutex<Vec<PersistedServer>>>,
    current_ip: Arc<Mutex<Option<String>>>,
    connection: Arc<Mutex<ServerConnectionState>>,
    session: Arc<Mutex<SessionState>>,
    app_handle: Option<AppHandle>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            servers: Arc::new(Mutex::new(Vec::new())),
            current_ip: Arc::new(Mutex::new(None)),
            connection: Arc::new(Mutex::new(ServerConnectionState::Disconnected)),
            session: Arc::new(Mutex::new(SessionState::Idle)),
            app_handle: Some(app_handle),
        }
    }

    // Getters
    pub async fn get_all_servers(&self) -> Vec<PersistedServer> {
        self.servers.lock().await.clone()
    }

    pub async fn get_current_ip(&self) -> Option<String> {
        self.current_ip.lock().await.clone()
    }

    pub async fn get_connection_state(&self) -> ServerConnectionState {
        self.connection.lock().await.clone()
    }

    pub async fn get_session_state(&self) -> SessionState {
        self.session.lock().await.clone()
    }

    // Setters
    pub async fn set_connection_ip(&self, ip: String) {
        *self.current_ip.lock().await = Some(ip);
    }

    pub async fn set_connection_state(&self, state: ServerConnectionState) {
        *self.connection.lock().await = state;
    }

    pub async fn set_session_state(&self, state: SessionState) {
        *self.session.lock().await = state;
    }

    pub async fn load_persisted_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let handle = self.app_handle.as_ref().ok_or("AppHandle not initialized")?;

        let stronghold = handle.state::<tauri_plugin_stronghold::stronghold::Stronghold>();
        let client = stronghold.load_client(CLIENT_NAME)?;
        let store = client.store();

        if let Ok(Some(servers_bytes)) = store.get("servers_list".as_bytes()) {
            let servers_vec: Vec<PersistedServer> = serde_json::from_slice(&servers_bytes)?;
            *self.servers.lock().await = servers_vec;
        }

        Ok(())
    }

    pub async fn save_servers(&self) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref handle) = self.app_handle {
        let servers = self.servers.lock().await.clone();
        
        let stronghold = handle.state::<tauri_plugin_stronghold::stronghold::Stronghold>();
        let client = stronghold.load_client(CLIENT_NAME)?;
        let store = client.store();

        let data = serde_json::to_vec(&servers)?;

        store.insert("servers_list".to_string().into_bytes(), data, None)?;

        stronghold.save()?;
    }
    Ok(())
}

    pub async fn add_server(
        &self,
        ip: String,
        server_name: String,
        user_key: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
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
            user_nickname: None,
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

    pub async fn save_nickname(
        &self,
        ip: &str,
        nickname: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        self.servers
            .lock()
            .await
            .iter()
            .find(|s| s.ip == ip)
            .map(|s| s.user_nickname.clone().unwrap_or_default())
    }

    pub async fn get_server(&self, ip: &str) -> Option<PersistedServer> {
        self.servers
            .lock()
            .await
            .iter()
            .find(|s| s.ip == ip)
            .cloned()
    }

    pub async fn get_server_by_id(&self, id: &str) -> Option<PersistedServer> {
        self.servers
            .lock()
            .await
            .iter()
            .find(|s| s.id == id)
            .cloned()
    }
}
