use crate::colors::*;
use crate::config::ServerConfig;
use crate::websocket::handle_websocket_upgrade;
use std::fs;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn handle_connection(mut stream: TcpStream, config: ServerConfig) {
    // Increased buffer size to handle large HTTP headers
    const BUFFER_SIZE: usize = 8192;
    const MAX_HEADER_SIZE: usize = 32768; // Maximum header size we'll accept

    let mut buffer = Vec::new();
    let mut temp_buf = [0; BUFFER_SIZE];
    let mut header_end_found = false;

    // Read data until we find the header end marker or exceed max size
    while buffer.len() < MAX_HEADER_SIZE {
        match stream.read(&mut temp_buf) {
            Ok(0) => break, // Connection closed
            Ok(bytes_read) => {
                // Add the newly read bytes to our buffer
                buffer.extend_from_slice(&temp_buf[..bytes_read]);

                // Check if we've found the end of headers
                if buffer.len() >= 4 {
                    let buf_slice = &buffer[buffer.len().saturating_sub(bytes_read + 4)..];
                    if buf_slice.windows(4).any(|window| window == b"\r\n\r\n") {
                        header_end_found = true;
                        break;
                    }
                }

                // If we've read less than the buffer size, we might be done
                if bytes_read < BUFFER_SIZE {
                    break;
                }
            }
            Err(_) => return, // Error reading from stream
        }
    }

    // If we didn't find the header end and exceeded max size, return error
    if !header_end_found && buffer.len() >= MAX_HEADER_SIZE {
        send_error_response(&mut stream, 431, "Request Header Fields Too Large");
        return;
    }

    let request = String::from_utf8_lossy(&buffer);
    let request_line = request.lines().next().unwrap_or("");

    if is_websocket_upgrade(&request) {
        if let Some(path) = parse_request_path(request_line) {
            if path == "/__live_reload_ws__" {
                handle_websocket_upgrade(stream, &request);
                return;
            }
        }
    }

    if let Some(path) = parse_request_path(request_line) {
        match path.as_str() {
            "/__live_reload__" => handle_live_reload(&mut stream),
            "/config.js" => serve_runtime_config(&mut stream, &config.ws_url),
            _ => serve_file(&mut stream, &config, &path),
        }
    } else {
        send_error_response(&mut stream, 400, "Bad Request");
    }
}

fn is_websocket_upgrade(request: &str) -> bool {
    let lines: Vec<&str> = request.lines().collect();
    let mut has_upgrade = false;
    let mut has_connection = false;
    let mut has_websocket_key = false;

    for line in lines {
        let lower = line.to_lowercase();
        if lower.starts_with("upgrade:") && lower.contains("websocket") {
            has_upgrade = true;
        } else if lower.starts_with("connection:") && lower.contains("upgrade") {
            has_connection = true;
        } else if lower.starts_with("sec-websocket-key:") {
            has_websocket_key = true;
        }
    }

    has_upgrade && has_connection && has_websocket_key
}

fn parse_request_path(request_line: &str) -> Option<String> {
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() >= 2 && parts[0] == "GET" {
        let path = parts[1].split('?').next().unwrap_or(parts[1]);
        Some(path.to_string())
    } else {
        None
    }
}

fn handle_live_reload(stream: &mut TcpStream) {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let last_change = crate::watcher::get_last_change();

    let response_body =
        format!(r#"{{"lastChange": {last_change}, "currentTime": {current_time}}}"#);
    send_response(
        stream,
        200,
        "OK",
        "application/json",
        response_body.as_bytes(),
        true,
    );
}

fn serve_file(stream: &mut TcpStream, config: &ServerConfig, requested_path: &str) {
    let mut file_path = config.root_dir.to_path_buf();

    let clean_path = requested_path.trim_start_matches('/');
    if clean_path.is_empty() {
        file_path.push("index.html");
    } else {
        file_path.push(clean_path);
    }

    // Security: Prevent directory traversal
    if !file_path.starts_with(&config.root_dir) {
        send_error_response(stream, 403, "Forbidden");
        return;
    }

    if file_path.is_dir() {
        file_path.push("index.html");
    }

    match fs::read(&file_path) {
        Ok(mut contents) => {
            if config.live_reload && is_html_file(&file_path) {
                contents = inject_live_reload_script(contents);
            }

            let mime_type = get_mime_type(&file_path);
            send_response(stream, 200, "OK", mime_type, &contents, false);
            println!("{BLUE}📄 {requested_path} - {GREEN}200 OK{RESET}");
        }
        Err(_) => {
            send_error_response(stream, 404, "Not Found");
            println!("{RED}❌ {requested_path} - {YELLOW}404 Not Found{RESET}");
        }
    }
}

fn serve_runtime_config(stream: &mut TcpStream, ws_url: &Option<String>) {
    let escaped_ws_url = ws_url
        .as_ref()
        .map(|url| url.replace('\\', "\\\\").replace('\'', "\\'"))
        .unwrap_or_default();
    let body = format!("window.__RUNTIME_CONFIG__ = {{ WS_URL: \\'{escaped_ws_url}\\' }}\n");
    send_response(
        stream,
        200,
        "OK",
        "application/javascript",
        body.as_bytes(),
        true,
    );
}

fn is_html_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("html") | Some("htm")
    )
}

fn inject_live_reload_script(contents: Vec<u8>) -> Vec<u8> {
    let content_str = String::from_utf8_lossy(&contents);
    let live_reload_script = get_live_reload_script();

    if let Some(head_pos) = content_str.find("</head>") {
        let mut new_content = content_str.to_string();
        new_content.insert_str(head_pos, &live_reload_script);
        new_content.into_bytes()
    } else if let Some(body_pos) = content_str.find("</body>") {
        let mut new_content = content_str.to_string();
        new_content.insert_str(body_pos, &live_reload_script);
        new_content.into_bytes()
    } else {
        contents
    }
}

fn get_live_reload_script() -> String {
    r#"
<script>
(function() {
    let lastChange = 0;
    let ws = null;
    let usePolling = false;
    let reconnectAttempts = 0;
    const maxReconnectAttempts = 5;

    function log(message) {
        console.log('🔄 Live Reload:', message);
    }

    function connectWebSocket() {
        if (usePolling) return;

        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/__live_reload_ws__`;

        try {
            ws = new WebSocket(wsUrl);

            ws.onopen = function() {
                log('Connected via WebSocket');
                reconnectAttempts = 0;
            };

            ws.onmessage = function(event) {
                const data = JSON.parse(event.data);
                if (data.type === 'reload') {
                    log('Files changed, reloading...');
                    window.location.reload();
                }
            };

            ws.onclose = function() {
                if (reconnectAttempts < maxReconnectAttempts) {
                    reconnectAttempts++;
                    setTimeout(() => {
                        log(`Reconnecting... (${reconnectAttempts}/${maxReconnectAttempts})`);
                        connectWebSocket();
                    }, 1000 * reconnectAttempts);
                } else {
                    log('WebSocket failed, falling back to polling');
                    usePolling = true;
                    startPolling();
                }
            };

            ws.onerror = function() {
                if (reconnectAttempts === 0) {
                    log('WebSocket error, falling back to polling');
                    usePolling = true;
                    startPolling();
                }
            };
        } catch (e) {
            log('WebSocket not supported, using polling');
            usePolling = true;
            startPolling();
        }
    }

    function startPolling() {
        function checkForChanges() {
            fetch('/__live_reload__')
                .then(response => response.json())
                .then(data => {
                    if (lastChange === 0) {
                        lastChange = data.lastChange;
                    } else if (data.lastChange > lastChange) {
                        log('Files changed, reloading...');
                        window.location.reload();
                    }
                })
                .catch(err => {
                    // Silently ignore errors
                });
        }

        setInterval(checkForChanges, 500);
        log('Using polling mode');
    }

    connectWebSocket();
})();
</script>
"#
    .to_string()
}

fn send_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    content_type: &str,
    body: &[u8],
    no_cache: bool,
) {
    let cache_header = if no_cache {
        "Cache-Control: no-cache\r\n"
    } else {
        "Cache-Control: public, max-age=3600\r\n"
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\n\
        Content-Type: {}\r\n\
        Content-Length: {}\r\n\
        Access-Control-Allow-Origin: *\r\n\
        {}\r\n",
        status_code,
        status_text,
        content_type,
        body.len(),
        cache_header
    );

    if stream.write_all(response.as_bytes()).is_ok() {
        if let Err(e) = stream.write_all(body) {
            eprintln!("Failed to write response body: {e}");
        }
    } else {
        eprintln!("Failed to write response headers");
    }
}

fn send_error_response(stream: &mut TcpStream, status_code: u16, status_text: &str) {
    let body = format!(
        "<!DOCTYPE html>\
        <html><head><title>{status_code} {status_text}</title></head>\
        <body><h1>{status_code} {status_text}</h1></body></html>"
    );

    send_response(
        stream,
        status_code,
        status_text,
        "text/html",
        body.as_bytes(),
        true,
    );
}

fn get_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("eot") => "application/vnd.ms-fontobject",
        Some("xml") => "application/xml",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("tar") => "application/x-tar",
        Some("gz") => "application/gzip",
        Some("txt") => "text/plain; charset=utf-8",
        Some("md") => "text/markdown; charset=utf-8",
        Some("csv") => "text/csv; charset=utf-8",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        _ => "application/octet-stream",
    }
}
