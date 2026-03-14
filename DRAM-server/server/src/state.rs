use std::sync::Arc;
use tokio::sync::Mutex;


#[derive(Clone)]
pub struct AppState {
    pub users: Arc<Mutex<Vec<crate::api::user_routes::User>>>,
    pub secret: String,
}