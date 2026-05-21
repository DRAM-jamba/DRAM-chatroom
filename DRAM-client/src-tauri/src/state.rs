use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;
use uuid::Uuid;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use rand::Rng;

use crate::client::WsClient;
use crate::models::PersistedServer;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a test server
    fn create_test_server(id: &str, ip: &str, name: &str, key: &str) -> PersistedServer {
        PersistedServer {
            id: id.to_string(),
            ip: ip.to_string(),
            server_name: name.to_string(),
            user_key: key.to_string(),
            user_nickname: None,
        }
    }

    #[test]
    fn test_server_creation_basics() {
        let server = create_test_server("id1", "127.0.0.1:8080", "TestServer", "key123");
        assert_eq!(server.id, "id1");
        assert_eq!(server.ip, "127.0.0.1:8080");
        assert_eq!(server.server_name, "TestServer");
        assert_eq!(server.user_key, "key123");
        assert_eq!(server.user_nickname, None);
    }

    #[test]
    fn test_server_with_nickname() {
        let mut server = create_test_server("id1", "127.0.0.1:8080", "TestServer", "key123");
        server.user_nickname = Some("MyServer".to_string());
        assert_eq!(server.user_nickname, Some("MyServer".to_string()));
    }

    #[test]
    fn test_duplicate_detection_logic() {
        let servers = vec![
            create_test_server("id1", "127.0.0.1:8080", "Server1", "key1"),
            create_test_server("id2", "127.0.0.1:8081", "Server2", "key2"),
        ];

        let ip_to_check = "127.0.0.1:8080";
        let has_duplicate = servers.iter().any(|s| s.ip == ip_to_check);
        assert!(has_duplicate);

        let ip_to_check = "999.999.999.999:9999";
        let has_duplicate = servers.iter().any(|s| s.ip == ip_to_check);
        assert!(!has_duplicate);
    }

    #[test]
    fn test_server_removal_logic() {
        let mut servers = vec![
            create_test_server("id1", "127.0.0.1:8080", "Server1", "key1"),
            create_test_server("id2", "127.0.0.1:8081", "Server2", "key2"),
            create_test_server("id3", "127.0.0.1:8082", "Server3", "key3"),
        ];

        servers.retain(|s| s.ip != "127.0.0.1:8081");

        assert_eq!(servers.len(), 2);
        assert!(servers.iter().all(|s| s.ip != "127.0.0.1:8081"));
    }

    #[test]
    fn test_server_search_by_ip() {
        let servers = vec![
            create_test_server("id1", "127.0.0.1:8080", "Server1", "key1"),
            create_test_server("id2", "127.0.0.1:8081", "Server2", "key2"),
            create_test_server("id3", "127.0.0.1:8082", "Server3", "key3"),
        ];

        let found = servers.iter().find(|s| s.ip == "127.0.0.1:8081");
        assert!(found.is_some());
        assert_eq!(found.unwrap().server_name, "Server2");

        let not_found = servers.iter().find(|s| s.ip == "999.999.999.999:9999");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_nickname_update_logic() {
        let mut servers = vec![create_test_server(
            "id1",
            "127.0.0.1:8080",
            "Server1",
            "key1",
        )];

        if let Some(server) = servers.iter_mut().find(|s| s.ip == "127.0.0.1:8080") {
            server.user_nickname = Some("UpdatedNickname".to_string());
        }

        assert_eq!(
            servers[0].user_nickname,
            Some("UpdatedNickname".to_string())
        );
    }

    #[test]
    fn test_nickname_retrieval_logic() {
        let mut server = create_test_server("id1", "127.0.0.1:8080", "Server1", "key1");
        server.user_nickname = Some("TestNick".to_string());

        let servers = vec![server];

        let nickname = servers
            .iter()
            .find(|s| s.ip == "127.0.0.1:8080")
            .map(|s| s.user_nickname.clone().unwrap_or_default());

        assert_eq!(nickname, Some("TestNick".to_string()));
    }

    #[test]
    fn test_nickname_retrieval_when_none() {
        let server = create_test_server("id1", "127.0.0.1:8080", "Server1", "key1");
        let servers = vec![server];

        let nickname = servers
            .iter()
            .find(|s| s.ip == "127.0.0.1:8080")
            .map(|s| s.user_nickname.clone().unwrap_or_default());

        // Returns empty string for None
        assert_eq!(nickname, Some(String::new()));
    }

    #[test]
    fn test_connection_state_enum() {
        let state1 = ServerConnectionState::Connected;
        let state2 = ServerConnectionState::Disconnected;

        match state1 {
            ServerConnectionState::Connected => assert!(true),
            ServerConnectionState::Disconnected => panic!("Expected Connected"),
        }

        match state2 {
            ServerConnectionState::Disconnected => assert!(true),
            ServerConnectionState::Connected => panic!("Expected Disconnected"),
        }
    }

    #[test]
    fn test_session_state_enum() {
        let state = SessionState::Idle;
        match state {
            SessionState::Idle => assert!(true),
            SessionState::JoinedSession(_) => panic!("Expected Idle"),
        }
    }

    #[test]
    fn test_persisted_server_equality() {
        let server1 = create_test_server("id1", "127.0.0.1:8080", "Server1", "key1");
        let server2 = create_test_server("id1", "127.0.0.1:8080", "Server1", "key1");
        let server3 = create_test_server("id2", "127.0.0.1:8081", "Server2", "key2");

        assert_eq!(server1, server2);
        assert_ne!(server1, server3);
    }

    #[test]
    fn test_server_filtering_multiple() {
        let servers = vec![
            create_test_server("id1", "192.168.1.1:8080", "Server1", "key1"),
            create_test_server("id2", "192.168.1.2:8080", "Server2", "key2"),
            create_test_server("id3", "192.168.1.3:8080", "Server3", "key3"),
        ];

        let filtered: Vec<_> = servers
            .iter()
            .filter(|s| s.ip.starts_with("192.168.1.1"))
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].ip, "192.168.1.1:8080");
    }

    #[test]
    fn test_all_servers_cloned() {
        let servers = vec![
            create_test_server("id1", "127.0.0.1:8080", "Server1", "key1"),
            create_test_server("id2", "127.0.0.1:8081", "Server2", "key2"),
        ];

        let cloned = servers.clone();
        assert_eq!(servers, cloned);
        assert_eq!(servers.len(), cloned.len());
    }

    #[test]
    fn test_server_list_iteration() {
        let servers = vec![
            create_test_server("id1", "127.0.0.1:8080", "Server1", "key1"),
            create_test_server("id2", "127.0.0.1:8081", "Server2", "key2"),
            create_test_server("id3", "127.0.0.1:8082", "Server3", "key3"),
        ];

        let ips: Vec<_> = servers.iter().map(|s| s.ip.clone()).collect();
        assert_eq!(ips.len(), 3);
        assert_eq!(ips[0], "127.0.0.1:8080");
    }

    #[test]
    fn test_server_ip_parsing() {
        let ip_str = "192.168.1.100:9000";
        let parts: Vec<&str> = ip_str.split(':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "192.168.1.100");
        assert_eq!(parts[1], "9000");
    }

    #[test]
    fn test_empty_server_list() {
        let servers: Vec<PersistedServer> = Vec::new();
        assert_eq!(servers.len(), 0);

        let found = servers.iter().find(|s| s.ip == "127.0.0.1:8080");
        assert!(found.is_none());
    }
}