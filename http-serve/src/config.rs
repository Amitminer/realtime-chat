//! Configuration management for the HTTP server.
//!
//! This module handles parsing configuration from both command-line arguments
//! and environment variables, with CLI arguments taking precedence.
//!
//! Configuration options:
//! - Host and port to bind to
//! - Root directory for serving files
//! - Live reload enable/disable
//! - WebSocket URL for runtime configuration

use std::env;
use std::path::PathBuf;

/// Configuration for the HTTP server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host to bind to (e.g., "0.0.0.0" or "127.0.0.1")
    pub host: String,
    /// Port to bind to (e.g., 8080)
    pub port: u16,
    /// Root directory for serving static files
    pub root_dir: PathBuf,
    /// Whether to enable live reload functionality
    pub live_reload: bool,
    /// WebSocket URL for runtime configuration (exposed to frontend)
    pub ws_url: Option<String>,
}

impl ServerConfig {
    /// Create a new ServerConfig from CLI arguments and environment variables.
    ///
    /// CLI arguments take precedence over environment variables.
    /// Supported CLI arguments:
    /// - `--port <port>`: Set the port to listen on
    /// - `--host <host>`: Set the host to bind to
    /// - `--root <path>`: Set the root directory for serving files
    /// - `--no-live-reload`: Disable live reload functionality
    /// - `--ws-url <url>`: Set the WebSocket URL for runtime configuration
    ///
    /// Environment variables:
    /// - `FRONTEND_HOST`: Host to bind to (default: "0.0.0.0")
    /// - `FRONTEND_PORT`: Port to bind to (default: 8080)
    /// - `ROOT_DIR`: Root directory for serving files (default: "frontend")
    /// - `LIVE_RELOAD`: Enable live reload (default: true, unless in Docker)
    /// - `WS_URL`: WebSocket URL for runtime configuration
    pub fn new() -> Self {
        // Load .env if present before reading env vars
        let _ = dotenvy::dotenv();

        let args: Vec<String> = env::args().collect();
        
        // Parse CLI options first
        let mut cli_port = None;
        let mut cli_host = None;
        let mut cli_root_dir = None;
        let mut cli_live_reload = None;
        let mut cli_ws_url = None;
        
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--port" => {
                    if i + 1 < args.len() {
                        cli_port = args[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "--host" => {
                    if i + 1 < args.len() {
                        cli_host = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--root" => {
                    if i + 1 < args.len() {
                        cli_root_dir = Some(PathBuf::from(&args[i + 1]));
                        i += 1;
                    }
                }
                "--no-live-reload" => {
                    cli_live_reload = Some(false);
                }
                "--ws-url" => {
                    if i + 1 < args.len() {
                        cli_ws_url = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                arg if !arg.starts_with("--") => {
                    cli_root_dir = Some(PathBuf::from(arg));
                }
                _ => {}
            }
            i += 1;
        }

        // Build final config with CLI taking precedence over env vars
        let port = cli_port.unwrap_or_else(|| {
            env::var("FRONTEND_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(8080)
        });
        
        let host = cli_host.unwrap_or_else(|| {
            env::var("FRONTEND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string())
        });
        
        let root_dir = cli_root_dir.unwrap_or_else(|| {
            env::var("ROOT_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("frontend"))
        });
        
        let live_reload = cli_live_reload.unwrap_or_else(|| {
            env::var("LIVE_RELOAD")
                .map(|v| v != "0" && v.to_lowercase() != "false")
                .unwrap_or(true)
        });
        
        let ws_url = cli_ws_url.or_else(|| {
            env::var("WS_URL").ok()
        });

        ServerConfig { host, port, root_dir, live_reload, ws_url }
    }
}
