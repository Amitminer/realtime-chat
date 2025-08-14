use std::env;
use std::path::PathBuf;

/// Configuration for the HTTP server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub root_dir: PathBuf,
    pub live_reload: bool,
    pub ws_url: Option<String>,
}

impl ServerConfig {
    pub fn new() -> Self {
        // Load .env if present before reading env vars
        let _ = dotenvy::dotenv();

        let args: Vec<String> = env::args().collect();
        let mut port = env::var("FRONTEND_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(8080);
        let host = env::var("FRONTEND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let mut root_dir = env::var("ROOT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("frontend"));
        let mut live_reload = env::var("LIVE_RELOAD")
            .map(|v| v != "0" && v.to_lowercase() != "false")
            .unwrap_or(true);
        let mut ws_url = env::var("WS_URL").ok();

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--port" => {
                    if i + 1 < args.len() {
                        port = args[i + 1].parse().unwrap_or(port);
                        i += 1;
                    }
                }
                "--host" => {
                    if i + 1 < args.len() {
                        // Note: host from CLI not yet used; kept minimal per original
                        let _ = &args[i + 1];
                        i += 1;
                    }
                }
                "--root" => {
                    if i + 1 < args.len() {
                        root_dir = PathBuf::from(&args[i + 1]);
                        i += 1;
                    }
                }
                "--no-live-reload" => {
                    live_reload = false;
                }
                "--ws-url" => {
                    if i + 1 < args.len() {
                        ws_url = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                arg if !arg.starts_with("--") => {
                    root_dir = PathBuf::from(arg);
                }
                _ => {}
            }
            i += 1;
        }

        ServerConfig { host, port, root_dir, live_reload, ws_url }
    }
}
