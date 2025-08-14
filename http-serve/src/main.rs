mod colors;
mod config;
mod handlers;
mod server;
mod watcher;
mod websocket;

use crate::colors::*;
use crate::config::ServerConfig;

fn main() {
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
