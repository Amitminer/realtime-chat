//! WebSocket server loop and connection handling.
//! Performs authentication and broadcasts AES-GCM encrypted messages.
use crate::config::Config;
use crate::encryption::EncryptionManager;
use crate::types::{
    AuthResponse, ChatMessage, EncryptedChatMessage, PasswordAuth, PeerMap, UserJoin,
};
use futures_util::{SinkExt, StreamExt};
use std::{collections::HashMap, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

/// Start the WebSocket server and process incoming connections.
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
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

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
                    let success = auth.password == server_password;
                    let response = AuthResponse {
                        success,
                        message: if success {
                            "Authentication successful".into()
                        } else {
                            "Invalid password".into()
                        },
                    };
                    if let Ok(s) = serde_json::to_string(&response) {
                        let _ = tx.send(Message::Text(s.into()));
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
            if let Ok(join) = serde_json::from_str::<UserJoin>(text) {
                if join.message_type == "join" {
                    username = join.username;
                    let (encrypted_message, nonce, _salt) = match encryption_manager
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
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        message_type: "join".into(),
                    };
                    let _ = broadcast_encrypted_message(&peer_map, &msg).await;
                    continue;
                }
            }

            // Encrypted message
            if let Ok(mut enc_msg) = serde_json::from_str::<EncryptedChatMessage>(text) {
                enc_msg.user_id = user_id.clone();
                enc_msg.username = username.clone();
                enc_msg.timestamp = chrono::Utc::now().to_rfc3339();
                enc_msg.message_type = "message".into();
                let _ = broadcast_encrypted_message(&peer_map, &enc_msg).await;
            } else if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(text) {
                let (encrypted_message, nonce, _salt) = match encryption_manager
                    .as_ref()
                    .encrypt_message(&chat_msg.message)
                {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let enc_msg = EncryptedChatMessage {
                    user_id: user_id.clone(),
                    username: username.clone(),
                    encrypted_message,
                    nonce,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    message_type: "message".into(),
                };
                let _ = broadcast_encrypted_message(&peer_map, &enc_msg).await;
            } else {
                let (encrypted_message, nonce, _salt) =
                    match encryption_manager.as_ref().encrypt_message(text) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                let enc_msg = EncryptedChatMessage {
                    user_id: user_id.clone(),
                    username: username.clone(),
                    encrypted_message,
                    nonce,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    message_type: "message".into(),
                };
                let _ = broadcast_encrypted_message(&peer_map, &enc_msg).await;
            }
        } else if msg.is_close() {
            break;
        }
    }

    if is_authenticated {
        if let Ok((encrypted_message, nonce, _salt)) = encryption_manager
            .as_ref()
            .encrypt_message(&format!("{username} left the chat"))
        {
            let leave_msg = EncryptedChatMessage {
                user_id: user_id.clone(),
                username: username.clone(),
                encrypted_message,
                nonce,
                timestamp: chrono::Utc::now().to_rfc3339(),
                message_type: "leave".into(),
            };
            let _ = broadcast_encrypted_message(&peer_map, &leave_msg).await;
        }
    }

    let mut peers = peer_map.write().await;
    peers.remove(&user_id);
}

/// Broadcast an encrypted message to all connected peers.
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
        if tx.send(msg.clone()).is_ok() {
            ok += 1;
        }
    }
    ok
}
