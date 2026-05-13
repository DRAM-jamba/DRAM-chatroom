use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;
use uuid::Uuid;
use tauri::Manager;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use rand::Rng;

use crate::client::WsClient;
use crate::models::PersistedServer;

pub const CLIENT_NAME: &str = "secure_storage";

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
    master_key: [u8; 32],
    servers: Arc<Mutex<Vec<PersistedServer>>>,
    current_ip: Arc<Mutex<Option<String>>>,
    connection: Arc<Mutex<ServerConnectionState>>,
    session: Arc<Mutex<SessionState>>,
    app_handle: Option<AppHandle>,
}

impl AppState {
    pub fn new(app_handle: AppHandle, master_key: [u8; 32]) -> Self {
        Self {
            master_key,
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
        use tauri_plugin_store::StoreExt;

        let handle = self.app_handle.as_ref().ok_or("AppHandle not initialized")?;
        let store = handle.store("app_data.json")?;

        let Some(val) = store.get("servers_list") else {
            println!("[state] No servers_list found in store");
            return Ok(());
        };

        let hex_str = val.as_str().ok_or("servers_list value is not a string")?;
        let blob = hex::decode(hex_str)?;

        if blob.len() < 12 {
            return Err("Stored blob is too short to contain a nonce".into());
        }

        let (nonce_bytes, ciphertext) = blob.split_at(12);
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decrypt error: {}", e))?;

        let servers_vec = serde_json::from_slice::<Vec<PersistedServer>>(&plaintext)?;
        println!("[state] Loaded {} servers from store", servers_vec.len());
        *self.servers.lock().await = servers_vec;

        Ok(())
    }

    pub async fn save_servers(&self) -> Result<(), Box<dyn std::error::Error>> {
        use tauri_plugin_store::StoreExt;

        let handle = self.app_handle.as_ref().ok_or("AppHandle not initialized")?;
        let servers = self.servers.lock().await.clone();
        let plaintext = serde_json::to_vec(&servers)?;

        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let nonce_bytes = rand::thread_rng().gen::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encrypt error: {}", e))?;

        let mut blob = nonce_bytes.to_vec();
        blob.extend(ciphertext);

        let store = handle.store("app_data.json")?;
        store.set("servers_list", hex::encode(&blob));
        store.save()?;

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