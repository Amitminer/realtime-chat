//! HTTP server initialization and main loop.
//!
//! This module contains the core server logic including:
//! - TCP listener setup
//! - Startup information printing
//! - File watcher initialization (for live reload)
//! - Main connection handling loop

use crate::colors::*;
use crate::config::ServerConfig;
use crate::handlers::handle_connection;
use crate::watcher::start_file_watcher;
use std::net::TcpListener;
use std::thread;

/// Start the HTTP server and begin listening for connections.
///
/// This function:
/// 1. Binds to the configured host and port
/// 2. Prints startup information
/// 3. Starts the file watcher (if live reload is enabled)
/// 4. Enters the main connection handling loop
///
/// # Arguments
///
/// * `config` - The server configuration
pub fn run_server(config: ServerConfig) {
    let bind_addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&bind_addr).unwrap_or_else(|e| {
        eprintln!(
            "{RED}{BOLD} Failed to bind to {bind_addr}: {e}{RESET}"
        );
        std::process::exit(1);
    });

    print_startup_info(&config, &bind_addr);

    // Start file watcher if live reload is enabled
    if config.live_reload {
        let watch_dir = config.root_dir.clone();
        thread::spawn(move || {
            if let Err(e) = start_file_watcher(watch_dir) {
                eprintln!("{RED}{BOLD} File watcher error: {e:?}{RESET}");
            }
        });
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let config_clone = config.clone();
                thread::spawn(move || {
                    handle_connection(stream, config_clone);
                });
            }
            Err(e) => {
                eprintln!("{RED}{BOLD} Connection failed: {e}{RESET}");
            }
        }
    }

    println!("{DIM}Frontend server process finished.{RESET}");
}

/// Print startup information to the console.
///
/// Displays server configuration details including:
/// - Server URL
/// - Root directory
/// - Live reload status
/// - WebSocket URL (if configured)
///
/// # Arguments
///
/// * `config` - The server configuration
/// * `bind_addr` - The address the server is bound to
fn print_startup_info(config: &ServerConfig, bind_addr: &str) {
    println!(
        "{GREEN}🚀 Static server running at {BOLD}http://{bind_addr}{RESET}"
    );
    println!(
        "{}📁 Serving files from: {}{}{}",
        BLUE,
        BOLD,
        config.root_dir.display(),
        RESET
    );

    if config.live_reload {
        println!("{CYAN}🔄 Live reload enabled{RESET}");
    } else {
        println!("{YELLOW}⏸️  Live reload disabled{RESET}");
    }

    if let Some(ref ws_url) = config.ws_url {
        println!("{MAGENTA}🌐 WebSocket URL: {BOLD}{ws_url}{RESET}");
    }

    println!("{DIM}Press Ctrl+C to stop{RESET}");
    println!();
}
