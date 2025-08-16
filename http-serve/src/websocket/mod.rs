//! WebSocket handling for live reload functionality.
//!
//! This module implements a minimal WebSocket server for handling live reload
//! notifications. It supports the WebSocket handshake protocol and maintains
//! a collection of connected clients to broadcast reload messages.
//!
//! Features:
//! - WebSocket handshake handling
//! - Client connection management
//! - Broadcast messaging to all connected clients
//! - Automatic client cleanup on disconnect

use crate::colors::*;
use base64::{engine::general_purpose, Engine as _};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::protocol::Role;
use tungstenite::{Message, WebSocket};

type ClientId = u64;
type WebSocketStream = WebSocket<TcpStream>;

lazy_static::lazy_static! {
    static ref WEBSOCKET_CLIENTS: Arc<Mutex<HashMap<ClientId, WebSocketStream>>> =
        Arc::new(Mutex::new(HashMap::new()));
    static ref CLIENT_COUNTER: Arc<Mutex<ClientId>> = Arc::new(Mutex::new(0));
}

/// Handle a WebSocket upgrade request.
///
/// This function performs the WebSocket handshake and spawns a new thread
/// to handle the WebSocket connection.
///
/// # Arguments
///
/// * `stream` - The TCP stream for the connection
/// * `request` - The raw HTTP request as a string
pub fn handle_websocket_upgrade(stream: TcpStream, request: &str) {
    if let Ok(websocket) = perform_handshake(stream, request) {
        let client_id = {
            let mut counter = CLIENT_COUNTER.lock().unwrap();
            *counter += 1;
            *counter
        };

        println!(
            "{GREEN}🔌 WebSocket client {client_id} connected{RESET}"
        );

        {
            let mut clients = WEBSOCKET_CLIENTS.lock().unwrap();
            clients.insert(client_id, websocket);
        }

        thread::spawn(move || {
            handle_websocket_client(client_id);
        });
    }
}

/// Perform the WebSocket handshake.
///
/// This function validates the WebSocket upgrade request and sends the
/// appropriate response to establish the WebSocket connection.
///
/// # Arguments
///
/// * `stream` - The TCP stream for the connection
/// * `request` - The raw HTTP request as a string
///
/// # Returns
///
/// A Result containing the WebSocket stream if successful, or an error
fn perform_handshake(
    mut stream: TcpStream,
    request: &str,
) -> Result<WebSocketStream, Box<dyn std::error::Error>> {
    let websocket_key = extract_websocket_key(request).ok_or("Missing WebSocket key")?;

    let response_key = generate_response_key(&websocket_key);

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {response_key}\r\n\
        \r\n"
    );

    stream.write_all(response.as_bytes())?;
    Ok(WebSocket::from_raw_socket(stream, Role::Server, None))
}

/// Extract the WebSocket key from the HTTP upgrade request.
///
/// # Arguments
///
/// * `request` - The raw HTTP request as a string
///
/// # Returns
///
/// Some(String) with the WebSocket key if found, None otherwise
fn extract_websocket_key(request: &str) -> Option<String> {
    for line in request.lines() {
        if line.to_lowercase().starts_with("sec-websocket-key:") {
            return line.split(':').nth(1).map(|s| s.trim().to_string());
        }
    }
    None
}

/// Generate the WebSocket accept key.
///
/// This function implements the WebSocket handshake key generation algorithm
/// as specified in RFC 6455.
///
/// # Arguments
///
/// * `client_key` - The client's WebSocket key
///
/// # Returns
///
/// The generated accept key as a base64-encoded string
fn generate_response_key(client_key: &str) -> String {
    const WEBSOCKET_MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut hasher = Sha1::new();
    hasher.update(client_key.as_bytes());
    hasher.update(WEBSOCKET_MAGIC_STRING.as_bytes());
    let result = hasher.finalize();
    general_purpose::STANDARD.encode(result)
}

/// Handle a WebSocket client connection.
///
/// This function manages the lifecycle of a WebSocket client connection,
/// reading messages and handling ping/pong frames. The connection is
/// automatically cleaned up when closed or on error.
///
/// # Arguments
///
/// * `client_id` - The unique identifier for this client
fn handle_websocket_client(client_id: ClientId) {
    loop {
        // Take the websocket out of the map to release the lock while we read from it.
        let mut websocket = match WEBSOCKET_CLIENTS.lock().unwrap().remove(&client_id) {
            Some(ws) => ws,
            // If the client is not in the map, it was likely removed by another thread (e.g., broadcast).
            // We can terminate this handler thread.
            None => return,
        };

        // Block on `read` without holding the lock on the client map.
        let should_break = match websocket.read() {
            Ok(Message::Close(_)) => {
                println!(
                    "{YELLOW}🔌 WebSocket client {client_id} disconnected (close frame){RESET}"
                );
                true
            }
            Ok(Message::Ping(payload)) => {
                if websocket.send(Message::Pong(payload)).is_err() {
                    println!(
                        "{YELLOW}🔌 WebSocket client {client_id} disconnected (pong failed){RESET}"
                    );
                    true
                } else {
                    false
                }
            }
            Ok(_) => false, // Other messages are fine, continue loop.
            Err(_) => {
                println!(
                    "{YELLOW}🔌 WebSocket client {client_id} disconnected (error){RESET}"
                );
                true
            }
        };

        if should_break {
            // Don't re-insert the websocket; it will be removed after the loop.
            break;
        } else {
            // If the connection is still active, put the websocket back in the map
            // so other threads (like broadcast) can access it.
            WEBSOCKET_CLIENTS.lock().unwrap().insert(client_id, websocket);
        }
    }

    // Final removal of the client from the map.
    WEBSOCKET_CLIENTS.lock().unwrap().remove(&client_id);
}

/// Broadcast a reload message to all connected WebSocket clients.
///
/// This function sends a reload message to all currently connected WebSocket
/// clients and automatically cleans up any disconnected clients.
pub fn broadcast_reload_message() {
    let mut clients = WEBSOCKET_CLIENTS.lock().unwrap();
    let message = Message::Text(r#"{"type":"reload"}"#.to_string());

    let mut clients_to_remove = Vec::new();

    for (client_id, websocket) in clients.iter_mut() {
        if websocket.send(message.clone()).is_err() {
            clients_to_remove.push(*client_id);
        }
    }

    for client_id in clients_to_remove {
        clients.remove(&client_id);
        println!(
            "{DIM}🔌 Removed disconnected WebSocket client {client_id}{RESET}"
        );
    }
}

/// Get the current number of connected WebSocket clients.
///
/// # Returns
///
/// The number of currently connected WebSocket clients
pub fn get_client_count() -> usize {
    let clients = WEBSOCKET_CLIENTS.lock().unwrap();
    clients.len()
}
