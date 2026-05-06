use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub messages: Vec<ChatMessage>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub sessions: Arc<Mutex<HashMap<String, Session>>>,
    pub api_key: String,
    pub model: String,
}

impl AppState {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            api_key,
            model: "claude-sonnet-4-6".to_string(),
        }
    }
}
