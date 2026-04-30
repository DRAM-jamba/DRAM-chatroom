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
        format!("{}/server/add", self.base_url)
    }
    pub fn connect(&self, user_key: &str) -> String {
        format!("{}/server/connect/{}", self.base_url, user_key)
    }
    pub fn leave(&self) -> String {
        format!("{}/server/leave", self.base_url)
    }
    pub fn forget(&self, user_key: &str) -> String {
        format!("{}/server/forget/{}", self.base_url, user_key)
    }

    // User commands
    pub fn set_nickname(&self, user_key: &str, nickname: &str) -> String {
        format!("{}/server/set/nickname/{}/{}", self.base_url, user_key, nickname)
    }

    // Session commands
    pub fn session_list(&self, user_key: &str) -> String {
        format!("{}/session/list/{}", self.base_url, user_key)
    }
    pub fn create_session(&self, user_key: &str, name: &str) -> String {
        format!("{}/session/create/{}/{}", self.base_url, user_key, name)
    }
    pub fn add_session(&self, user_key: &str, session_key: &str) -> String {
        format!("{}/session/add/{}/{}", self.base_url, user_key, session_key)
    }
    pub fn connect_session(&self, key: &str) -> String {
        format!("{}/session/connect/{}", self.base_url, key)
    }
    pub fn leave_session(&self) -> String {
        format!("{}/session/leave", self.base_url)
    }
    pub fn delete_session(&self, user_key: &str, session_key: &str) -> String {
        format!("{}/session/delete/{}/{}", self.base_url, user_key, session_key)
    }
    pub fn forget_session(&self, user_key: &str, session_key: &str) -> String {
        format!("{}/session/forget/{}/{}", self.base_url, user_key, session_key)
    }

    // WebSocket URL
    pub fn ws(&self, user_key: &str) -> String {
        format!("{}/ws/{}", self.base_url.replace("http", "ws"), user_key)
    }
}