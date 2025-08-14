//! Shared message and state types used across the server.
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};

/// Sender for WebSocket text frames to a single client.
pub type Tx = mpsc::UnboundedSender<Message>;
/// Map of connected user id -> sender channel.
pub type PeerMap = Arc<RwLock<HashMap<String, Tx>>>;

/// Plaintext message representation (legacy / internal use only).
#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub user_id: String,
    pub username: String,
    pub message: String,
    pub timestamp: String,
    pub message_type: String,
}

/// Encrypted message envelope as sent to clients.
#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptedChatMessage {
    pub user_id: String,
    pub username: String,
    pub encrypted_message: String,
    pub nonce: String,
    pub timestamp: String,
    pub message_type: String,
}

/// Client request to join with a selected username.
#[derive(Serialize, Deserialize)]
pub struct UserJoin {
    pub username: String,
    pub message_type: String,
}

/// Client request to authenticate with a password.
#[derive(Serialize, Deserialize)]
pub struct PasswordAuth {
    pub password: String,
}

/// Server response to authentication attempt.
#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
}
