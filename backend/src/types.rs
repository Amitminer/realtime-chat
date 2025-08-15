//! Shared message and state types used across the server.
//!
//! This module defines the data structures used for communication between
//! the server and clients, as well as internal server state management.
//!
//! Message flow:
//! 1. Client sends PasswordAuth for authentication
//! 2. Server responds with AuthResponse
//! 3. Client sends UserJoin to set username
//! 4. Server broadcasts EncryptedChatMessage for join/leave/chat events

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::tungstenite::Message;

/// Sender for WebSocket text frames to a single client.
pub type Tx = mpsc::Sender<Message>;
/// Map of connected user id -> sender channel.
pub type PeerMap = Arc<RwLock<HashMap<String, Tx>>>;

/// Message type enum for all WebSocket communications.
///
/// This enum defines the different types of messages that can be exchanged
/// between the server and clients.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MessageType {
    /// Authentication request from client
    Auth,
    /// Authentication response from server
    AuthResponse,
    /// User join notification
    Join,
    /// User leave notification
    Leave,
    /// Chat message
    Chat,
    /// Encrypted chat message (deprecated)
    EncryptedChat,
    /// System message
    System,
    /// Username validation error
    UsernameError,
}

/// Plaintext message representation (legacy / internal use only).
///
/// This struct represents a plaintext chat message. It's primarily used
/// for internal processing before encryption.
#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    /// Unique identifier for the user
    pub user_id: String,
    /// Display name of the user
    pub username: String,
    /// The message content (plaintext)
    pub message: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Type of message
    pub message_type: MessageType,
}

/// Encrypted message envelope as sent to clients.
///
/// This struct represents an encrypted message that is broadcast to all
/// connected clients. The actual message content is encrypted and can
/// only be decrypted by clients with the correct password.
#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptedChatMessage {
    /// Unique identifier for the user
    pub user_id: String,
    /// Display name of the user
    pub username: String,
    /// Base64-encoded encrypted message content
    pub encrypted_message: String,
    /// Base64-encoded nonce used for encryption
    pub nonce: String,
    /// Base64-encoded salt used for key derivation
    pub salt: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Type of message
    pub message_type: MessageType,
}

/// Client request to join with a selected username.
///
/// This struct represents a client's request to join the chat with a
/// specific username.
#[derive(Serialize, Deserialize)]
pub struct UserJoin {
    /// The username the client wants to use
    pub username: String,
    /// Must be MessageType::Join
    pub message_type: MessageType,
}

/// Client request to authenticate with a password.
///
/// This struct represents a client's authentication request.
#[derive(Serialize, Deserialize)]
pub struct PasswordAuth {
    /// The password for authentication
    pub password: String,
    /// Must be MessageType::Auth
    pub message_type: MessageType,
}

/// Server response to authentication attempt.
///
/// This struct represents the server's response to a client's
/// authentication request.
#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    /// Whether authentication was successful
    pub success: bool,
    /// Human-readable message
    pub message: String,
    /// Must be MessageType::AuthResponse
    pub message_type: MessageType,
}

/// Server response for username validation errors.
///
/// This struct represents the server's response when a client's
/// username is invalid.
#[derive(Serialize, Deserialize)]
pub struct UsernameError {
    /// Always true for this message type
    pub error: bool,
    /// Human-readable error message
    pub message: String,
    /// Must be MessageType::UsernameError
    pub message_type: MessageType,
}
