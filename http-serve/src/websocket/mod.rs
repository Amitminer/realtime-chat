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

fn extract_websocket_key(request: &str) -> Option<String> {
    for line in request.lines() {
        if line.to_lowercase().starts_with("sec-websocket-key:") {
            return line.split(':').nth(1).map(|s| s.trim().to_string());
        }
    }
    None
}

fn generate_response_key(client_key: &str) -> String {
    const WEBSOCKET_MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut hasher = Sha1::new();
    hasher.update(client_key.as_bytes());
    hasher.update(WEBSOCKET_MAGIC_STRING.as_bytes());
    let result = hasher.finalize();
    general_purpose::STANDARD.encode(result)
}

fn handle_websocket_client(client_id: ClientId) {
    loop {
        let should_break = {
            let mut clients = WEBSOCKET_CLIENTS.lock().unwrap();

            if let Some(websocket) = clients.get_mut(&client_id) {
                match websocket.read() {
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
                    Ok(_) => false,
                    Err(_) => {
                        println!(
                            "{YELLOW}🔌 WebSocket client {client_id} disconnected (error){RESET}"
                        );
                        true
                    }
                }
            } else {
                true
            }
        };

        if should_break {
            break;
        }

        thread::sleep(std::time::Duration::from_millis(100));
    }

    let mut clients = WEBSOCKET_CLIENTS.lock().unwrap();
    clients.remove(&client_id);
}

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

pub fn get_client_count() -> usize {
    let clients = WEBSOCKET_CLIENTS.lock().unwrap();
    clients.len()
}
