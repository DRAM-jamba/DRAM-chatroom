pub struct ServerApi {
    base_url: String,
}

impl ServerApi {
    pub fn new(base_url: &str) -> Self {
        Self { base_url: base_url.to_string(), }
    }

    // Server endpoints
    pub fn add_server(&self) -> String {
        format!("{}/server/add", self.base_url)
    }
    pub fn connect_server(&self) -> String {
        format!("{}/server/connect", self.base_url)
    }
    pub fn leave_server(&self) -> String {
        format!("{}/server/leave", self.base_url)
    }
    pub fn forget_server(&self) -> String {
        format!("{}/server/forget", self.base_url)
    }
    pub fn set_nickname(&self) -> String {
        format!("{}/server/nickname", self.base_url)
    }

    // Session commands
    pub fn session_list(&self) -> String {
        format!("{}/session/list", self.base_url)
    }
    pub fn create_session(&self) -> String {
        format!("{}/session/create", self.base_url)
    }
    pub fn add_session(&self) -> String {
        format!("{}/session/add", self.base_url)
    }
    pub fn leave_session(&self) -> String {
        format!("{}/session/leave", self.base_url)
    }
    pub fn forget_session(&self) -> String {
        format!("{}/session/forget", self.base_url)
    }
    pub fn delete_session(&self) -> String {
        format!("{}/session/delete", self.base_url)
    }

    // WebSocket URL
    pub fn ws(&self, user_key: &str, session_key: &str) -> String {
        format!(
            "{}/session/connect/{}/{}",
            self.base_url.replace("http", "ws"),
            user_key,
            session_key
        )
    }
}
