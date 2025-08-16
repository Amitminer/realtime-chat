//! HTTP static file server with live reload capabilities.
//!
//! This crate implements a minimal HTTP server for serving static files,
//! with optional live reload functionality for development.
//!
//! Features:
//! - Static file serving with proper MIME types
//! - Live reload with WebSocket-based notifications
//! - Runtime configuration endpoint (`/config.js`)
//! - Security checks to prevent directory traversal
//! - CLI and environment variable configuration

mod colors;
mod config;
mod handlers;
mod server;
mod watcher;
mod websocket;

use crate::colors::*;
use crate::config::ServerConfig;

/// Print usage information for the HTTP server.
fn print_usage() {
    println!("Usage: http-serve [OPTIONS] [ROOT_DIR]");
    println!();
    println!("A minimal HTTP server for serving static files with live reload capabilities.");
    println!();
    println!("Positional arguments:");
    println!("  ROOT_DIR              Root directory for serving files (default: 'frontend')");
    println!();
    println!("Options:");
    println!("  --port <port>         Port to listen on (default: 8080)");
    println!("  --host <host>         Host to bind to (default: 0.0.0.0)");
    println!("  --root <path>         Root directory for serving files");
    println!("  --no-live-reload      Disable live reload functionality");
    println!("  --ws-url <url>        WebSocket URL for runtime configuration");
    println!("  --help, -h            Display this help message");
    println!();
    println!("Environment variables:");
    println!("  FRONTEND_HOST         Host to bind to (default: 0.0.0.0)");
    println!("  FRONTEND_PORT         Port to listen on (default: 8080)");
    println!("  ROOT_DIR              Root directory for serving files (default: 'frontend')");
    println!("  LIVE_RELOAD           Enable live reload (default: true)");
    println!("  WS_URL                WebSocket URL for runtime configuration");
    println!();
    println!("Examples:");
    println!("  http-serve                    # Serve files from 'frontend' directory on port 8080");
    println!("  http-serve --port 3000        # Serve on port 3000");
    println!("  http-serve --root ./public    # Serve files from 'public' directory");
    println!("  http-serve ./docs             # Serve files from 'docs' directory");
    println!("  http-serve --no-live-reload   # Serve without live reload");
}

/// Entry point for the HTTP static file server.
///
/// This function:
/// 1. Parses configuration from CLI arguments and environment variables
/// 2. Validates that the root directory exists
/// 3. Starts the HTTP server
fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check for help flags
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return;
    }

    let config = ServerConfig::new();

    if !config.root_dir.exists() {
        eprintln!(
            "{}{}Error:{} Directory '{}' does not exist{}",
            RED,
            BOLD,
            RESET,
            config.root_dir.display(),
            RESET
        );
        std::process::exit(1);
    }

    server::run_server(config);
}
