pub struct ServerApi {
    base_url: String,
}

impl ServerApi {
    // URL builders
    pub fn new(base_url: &str) -> Self {
        Self { base_url: base_url.to_string() }
    }

    // Server commands
    pub fn add(&self) -> String {
        format!("{}/add", self.base_url)
    }
    pub fn connect(&self, user_key: &str) -> String {
        format!("{}/connect/{}", self.base_url, user_key)
    }
    pub fn leave(&self) -> String {
        format!("{}/leave", self.base_url)
    }
    pub fn forget(&self, user_key: &str) -> String {
        format!("{}/forget/{}", self.base_url, user_key)
    }

    // User commands
    pub fn set_nickname(&self, user_key: &str, nickname: &str) -> String {
        format!("{}/set/nickname/{}/{}", self.base_url, user_key, nickname)
    }

    // Session commands
    pub fn create_session(&self, user_key: &str, name: &str) -> String {
        format!("{}/session/create/{}/{}", self.base_url, user_key, name)
    }
    pub fn join_session(&self, user_key: &str, key: &str) -> String {
        format!("{}/session/join/{}/{}", self.base_url, user_key, key)
    }
    pub fn leave_session(&self, user_key: &str) -> String {
        format!("{}/session/leave/{}", self.base_url, user_key)
    }
    pub fn delete_session(&self, user_key: &str, session_id: &str) -> String {
        format!("{}/session/delete/{}/{}", self.base_url, user_key, session_id)
    }

    // WebSocket URL
    pub fn ws(&self, user_key: &str) -> String {
        format!("{}/ws/{}", self.base_url.replace("http", "ws"), user_key)
    }
}