use crate::colors::*;
use crate::config::ServerConfig;
use crate::handlers::handle_connection;
use crate::watcher::start_file_watcher;
use std::net::TcpListener;
use std::thread;

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
