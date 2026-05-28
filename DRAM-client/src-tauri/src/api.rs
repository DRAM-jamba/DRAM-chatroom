use crate::error::AppError;
use crate::models::{ChallengeFromServer, ChallengeSolvePayload, ServerErrorPayload, TokenResponse};
use crate::security::derive_identity_keypair;
use ed25519_dalek::Signer;

pub struct ServerApi {
    base_url: String,
    token: Option<String>,
}

impl ServerApi {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            token: None,
        }
    }

    pub fn with_token(base_url: &str, token: String) -> Self {
        Self {
            base_url: base_url.to_string(),
            token: Some(token),
        }
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {}", t))
    }

    // Authentication endpoints
    pub fn challenge(&self) -> String {
        format!("{}/server/challenge", self.base_url)
    }
    pub fn token_url(&self) -> String {
        format!("{}/server/token", self.base_url)
    }
    pub fn refresh_token_url(&self) -> String {
        format!("{}/server/refresh_token", self.base_url)
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

    // Voice-chat
    pub fn create_voicechat(&self) -> String {
        format!("{}/session/token", self.base_url)
    }

    // WebSocket URL
    pub fn ws(&self) -> String {
        format!("{}/session/connect", self.base_url.replace("http", "ws"))
    }

    async fn inject_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.auth_header() {
            Some(header) => builder.header("Authorization", header),
            None => builder,
        }
    }

    pub async fn http_post_authed<B: serde::Serialize>(&self, url: &str, body: &B) -> Result<reqwest::Response, AppError> {
        let req = self.inject_auth(reqwest::Client::new().post(url).json(body)).await;
        Self::handle_response(req.send().await?).await
    }

    pub async fn http_put_authed<B: serde::Serialize>(&self, url: &str, body: &B) -> Result<reqwest::Response, AppError> {
        let req = self.inject_auth(reqwest::Client::new().put(url).json(body)).await;
        Self::handle_response(req.send().await?).await
    }

    pub async fn http_delete_authed<B: serde::Serialize>(&self, url: &str, body: &B) -> Result<reqwest::Response, AppError> {
        let req = self.inject_auth(reqwest::Client::new().delete(url).json(body)).await;
        Self::handle_response(req.send().await?).await
    }

    pub async fn http_get_authed<B: serde::Serialize>(&self, url: &str, body: &B) -> Result<reqwest::Response, AppError> {
        let req = self.inject_auth(reqwest::Client::new().get(url).json(body)).await;
        Self::handle_response(req.send().await?).await
    }

    pub async fn http_patch_authed<B: serde::Serialize>(&self, url: &str, body: &B) -> Result<reqwest::Response, AppError> {
        let req = self.inject_auth(reqwest::Client::new().patch(url).json(body)).await;
        Self::handle_response(req.send().await?).await
    }

    async fn handle_response(response: reqwest::Response) -> Result<reqwest::Response, AppError> {
        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        let error_msg = match serde_json::from_str::<ServerErrorPayload>(&error_text) {
            Ok(payload) => {
                if payload.error.contains("violates foreign key constraint") {
                    "The provided key does not exist or is invalid.".to_string()
                } else if payload.error.contains("violates unique constraint") {
                    "This entry already exists.".to_string()
                } else {
                    payload.error
                }
            }
            Err(_) => {
                if error_text.is_empty() {
                    "No error details provided by server".to_string()
                } else {
                    error_text
                }
            }
        };

        Err(AppError::Protocol(format!("{} (Status: {})", error_msg, status.as_u16())))
    }

    pub async fn http_post_empty(url: &str) -> Result<reqwest::Response, AppError> {
        let response = reqwest::Client::new()
            .post(url)
            .send()
            .await?;
            
        Self::handle_response(response).await
    }

    pub async fn http_post<B: serde::Serialize>(
        url: &str,
        body: &B,
    ) -> Result<reqwest::Response, AppError> {
        let response = reqwest::Client::new()
            .post(url)
            .json(body)
            .send()
            .await?;
            
        Self::handle_response(response).await
    }

    pub async fn http_put<B: serde::Serialize>(
        url: &str,
        body: &B,
    ) -> Result<reqwest::Response, AppError> {
        let response = reqwest::Client::new()
            .put(url)
            .json(body)
            .send()
            .await?;
            
        Self::handle_response(response).await
    }

    pub async fn http_delete_empty(url: &str) -> Result<reqwest::Response, AppError> {
        let response = reqwest::Client::new()
            .delete(url)
            .send()
            .await?;
            
        Self::handle_response(response).await
    }

    pub async fn http_delete<B: serde::Serialize>(
        url: &str,
        body: &B,
    ) -> Result<reqwest::Response, AppError> {
        let response = reqwest::Client::new()
            .delete(url)
            .json(body)
            .send()
            .await?;
            
        Self::handle_response(response).await
    }

    pub async fn http_get<B: serde::Serialize>(
        url: &str,
        body: &B,
    ) -> Result<reqwest::Response, AppError> {
        let response = reqwest::Client::new()
            .get(url)
            .json(body)
            .send()
            .await?;
            
        Self::handle_response(response).await
    }

    pub async fn http_patch<B: serde::Serialize>(
        url: &str,
        body: &B,
    ) -> Result<reqwest::Response, AppError> {
        let response = reqwest::Client::new()
            .patch(url)
            .json(body)
            .send()
            .await?;
            
        Self::handle_response(response).await
    }
}
