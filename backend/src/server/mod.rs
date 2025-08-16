//! WebSocket server loop and connection handling.
//!
//! This module implements the core WebSocket server functionality, including:
//! - Connection establishment and authentication
//! - Message broadcasting to all connected clients
//! - AES-GCM encryption for all messages
//! - Username validation and management
//!
//! The server follows a strict authentication flow where clients must first
//! authenticate with a password before they can join with a username and
//! participate in the chat.

use crate::config::Config;
use crate::encryption::EncryptionManager;
use crate::types::{
    AuthResponse, ChatMessage, EncryptedChatMessage, MessageType, PasswordAuth, PeerMap, UserJoin,
    UsernameError,
};
use constant_time_eq::constant_time_eq;
use futures_util::{SinkExt, StreamExt};
use std::{collections::HashMap, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

/// Start the WebSocket server and process incoming connections.
///
/// This function initializes the WebSocket server, binds to the configured
/// address, and enters the main connection handling loop.
///
/// # Arguments
///
/// * `config` - The server configuration containing bind address and password
///
/// # Returns
///
/// A Result indicating success or failure
pub async fn run_server(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let listener = match TcpListener::bind(&config.bind_addr).await {
        Ok(listener) => {
            println!(
                "✅ Server successfully bound to: ws://{}",
                &config.bind_addr
            );
            println!("🛡️  All messages will be encrypted end-to-end");
            println!("🔐 Password required for access");
            listener
        }
        Err(e) => {
            println!("❌ Failed to bind to {}: {}", &config.bind_addr, e);
            return Err(e.into());
        }
    };

    let peer_map: PeerMap = Arc::new(RwLock::new(HashMap::new()));

    // Create encryption manager with proper error handling
    let encryption_manager: Arc<EncryptionManager> =
        match EncryptionManager::new(&config.server_password) {
            Ok(manager) => Arc::new(manager),
            Err(e) => {
                eprintln!("❌ Failed to initialize encryption manager: {e}");
                return Err(e);
            }
        };

    println!("👂 Listening for encrypted connections...");

    let mut connection_count = 0;
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                connection_count += 1;
                println!(
                    "\n[DEBUG] 🔔 Encrypted connection #{connection_count} accepted from: {addr}"
                );

                let peer_map_clone = peer_map.clone();
                let enc_clone = encryption_manager.clone();
                let password = config.server_password.clone();
                tokio::spawn(async move {
                    handle_connection(peer_map_clone, stream, addr, enc_clone, &password).await;
                    println!("[DEBUG] 🏁 Encrypted connection handler finished for: {addr}");
                });
            }
            Err(e) => {
                eprintln!("[ERROR] accept() failed: {e}. Retrying in 500ms...");
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }
        }
    }
}

/// Handle a single WebSocket connection lifecycle.
///
/// This function manages the complete lifecycle of a WebSocket connection,
/// including authentication, message processing, and cleanup.
///
/// The connection flow is:
/// 1. Wait for password authentication
/// 2. Wait for username join
/// 3. Process chat messages
/// 4. Handle disconnect and notify other users
///
/// # Arguments
///
/// * `peer_map` - Shared map of connected clients
/// * `raw_stream` - The raw TCP stream for this connection
/// * `addr` - The remote address of the client
/// * `encryption_manager` - The encryption manager for message encryption
/// * `server_password` - The server password for authentication
async fn handle_connection(
    peer_map: PeerMap,
    raw_stream: TcpStream,
    addr: std::net::SocketAddr,
    encryption_manager: Arc<EncryptionManager>,
    server_password: &str,
) {
    println!("[DEBUG] 🔗 Incoming TCP connection from: {addr}");

    let ws_stream = match accept_async(raw_stream).await {
        Ok(stream) => {
            println!("[DEBUG] ✅ WebSocket handshake successful for: {addr}");
            stream
        }
        Err(e) => {
            println!("[ERROR] ❌ WebSocket handshake failed for {addr}: {e}");
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    // Create a bounded channel with a capacity of 100 messages
    let (tx, mut rx) = mpsc::channel::<Message>(100);

    let user_id = Uuid::new_v4().to_string();
    let mut username = format!("User_{}", &user_id[..8]);
    let mut is_authenticated = false;

    // Outgoing messages task
    let peer_map_clone = peer_map.clone();
    let user_id_clone = user_id.clone();
    let username_clone = username.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = ws_sender.send(msg).await {
                eprintln!("[ERROR] send to {user_id_clone} failed: {e}");
                break;
            }
        }
        let mut peers = peer_map_clone.write().await;
        peers.remove(&user_id_clone);
        println!("[DEBUG] removed user {username_clone}");
    });

    // Incoming loop
    while let Some(msg) = ws_receiver.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                eprintln!("recv error: {e}");
                break;
            }
        };

        if msg.is_text() {
            let text = msg.to_text().unwrap();

            // Auth first
            if !is_authenticated {
                if let Ok(auth) = serde_json::from_str::<PasswordAuth>(text) {
                    let success =
                        constant_time_eq(auth.password.as_bytes(), server_password.as_bytes());
                    let response = AuthResponse {
                        success,
                        message: if success {
                            "Authentication successful".into()
                        } else {
                            "Invalid password".into()
                        },
                        message_type: MessageType::AuthResponse,
                    };
                    if let Ok(s) = serde_json::to_string(&response)
                        && let Err(e) = tx.send(Message::Text(s.into())).await
                    {
                        eprintln!("[ERROR] Failed to send auth response to {addr}: {e}");
                        break;
                    }
                    if success {
                        is_authenticated = true;
                        let mut peers = peer_map.write().await;
                        peers.insert(user_id.clone(), tx.clone());
                    } else {
                        break;
                    }
                    continue;
                } else {
                    break;
                }
            }

            // Username join
            if let Ok(join) = serde_json::from_str::<UserJoin>(text)
                && join.message_type == MessageType::Join
            {
                // Validate username
                match validate_username(&join.username) {
                    Ok(()) => {
                        username = join.username;
                    }
                    Err(e) => {
                        // Send error message to client
                        let error_response = UsernameError {
                            error: true,
                            message: e,
                            message_type: MessageType::UsernameError,
                        };
                        if let Ok(s) = serde_json::to_string(&error_response)
                            && let Err(e) = tx.send(Message::Text(s.into())).await
                        {
                            eprintln!("[ERROR] Failed to send username error to {addr}: {e}");
                            break;
                        }
                        continue;
                    }
                }

                let (encrypted_message, nonce, salt) = match encryption_manager
                        .as_ref()
                        .encrypt_message(&format!("{username} joined the chat"))
                    {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                let msg = EncryptedChatMessage {
                    user_id: user_id.clone(),
                    username: username.clone(),
                    encrypted_message,
                    nonce,
                    salt,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    message_type: MessageType::Join,
                };
                let _ = broadcast_encrypted_message(&peer_map, &msg).await;
                continue;
            }

            // Encrypted message
            if let Ok(mut enc_msg) = serde_json::from_str::<EncryptedChatMessage>(text) {
                enc_msg.user_id = user_id.clone();
                enc_msg.username = username.clone();
                enc_msg.timestamp = chrono::Utc::now().to_rfc3339();
                enc_msg.message_type = MessageType::Chat;
                let _ = broadcast_encrypted_message(&peer_map, &enc_msg).await;
            } else if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(text) {
                let (encrypted_message, nonce, salt) = match encryption_manager
                    .as_ref()
                    .encrypt_message(&chat_msg.message)
                {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Encryption failed: {e}");
                        continue;
                    }
                };
                let enc_msg = EncryptedChatMessage {
                    user_id: user_id.clone(),
                    username: username.clone(),
                    encrypted_message,
                    nonce,
                    salt,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    message_type: MessageType::Chat,
                };
                let _ = broadcast_encrypted_message(&peer_map, &enc_msg).await;
            } else {
                let (encrypted_message, nonce, salt) =
                    match encryption_manager.as_ref().encrypt_message(text) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Encryption failed: {e}");
                            continue;
                        }
                    };
                let enc_msg = EncryptedChatMessage {
                    user_id: user_id.clone(),
                    username: username.clone(),
                    encrypted_message,
                    nonce,
                    salt,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    message_type: MessageType::Chat,
                };
                let _ = broadcast_encrypted_message(&peer_map, &enc_msg).await;
            }
        } else if msg.is_close() {
            break;
        }
    }

    // Handle user leaving
    if is_authenticated {
        let encryption_result = encryption_manager
            .as_ref()
            .encrypt_message(&format!("{username} left the chat"));

        if let Ok((encrypted_message, nonce, salt)) = encryption_result {
            let leave_msg = EncryptedChatMessage {
                user_id: user_id.clone(),
                username: username.clone(),
                encrypted_message,
                nonce,
                salt,
                timestamp: chrono::Utc::now().to_rfc3339(),
                message_type: MessageType::Leave,
            };
            let _ = broadcast_encrypted_message(&peer_map, &leave_msg).await;
        }
    }

    // Clean up peer from map
    let mut peers = peer_map.write().await;
    peers.remove(&user_id);
}

/// Broadcast an encrypted message to all connected peers.
///
/// This function sends an encrypted message to all currently connected clients.
/// It uses a non-blocking send operation to prevent slow clients from blocking
/// the server.
///
/// # Arguments
///
/// * `peer_map` - Shared map of connected clients
/// * `encrypted_msg` - The encrypted message to broadcast
///
/// # Returns
///
/// The number of clients the message was successfully sent to
async fn broadcast_encrypted_message(
    peer_map: &PeerMap,
    encrypted_msg: &EncryptedChatMessage,
) -> usize {
    let peers = peer_map.read().await;
    let msg_text = match serde_json::to_string(encrypted_msg) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let msg = Message::Text(msg_text.into());
    let mut ok = 0;
    for (_user_id, tx) in peers.iter() {
        // Try to send the message, but don't block or break on failure
        // Slow clients will have their messages dropped if their buffer is full
        if tx.try_send(msg.clone()).is_ok() {
            ok += 1;
        }
    }
    ok
}

/// Validate a username for length, characters, and reserved names.
///
/// This function checks if a username meets the server's requirements:
/// - Not empty
/// - No longer than 20 characters
/// - Contains only alphanumeric characters, underscores, or hyphens
/// - Not a reserved name (system, admin, server, moderator)
///
/// # Arguments
///
/// * `username` - The username to validate
///
/// # Returns
///
/// Ok(()) if the username is valid, Err(message) if invalid
fn validate_username(username: &str) -> Result<(), String> {
    // Check length
    if username.is_empty() {
        return Err("Username cannot be empty".to_string());
    }

    if username.len() > 20 {
        return Err("Username too long (max 20 characters)".to_string());
    }

    // Check allowed characters (alphanumeric, underscore, hyphen)
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("Username contains invalid characters (only alphanumeric, underscore, and hyphen allowed)".to_string());
    }

    // Reserved names
    let reserved_names = ["system", "admin", "server", "moderator"];
    let lower_username = username.to_lowercase();
    if reserved_names.iter().any(|&name| lower_username == name) {
        return Err("Username is reserved".to_string());
    }

    Ok(())
}
