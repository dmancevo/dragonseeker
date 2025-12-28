// Shared application state - to be implemented

use crate::core::GameManager;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub game_manager: Arc<RwLock<GameManager>>,
    pub secret_key: String,
}

impl AppState {
    pub fn new(secret_key: String) -> Self {
        Self {
            game_manager: Arc::new(RwLock::new(GameManager::new())),
            secret_key,
        }
    }
}
