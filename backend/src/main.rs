//! Encrypted WebSocket chat server entrypoint.
//!
//! This is the main entry point for the encrypted WebSocket chat server.
//!
//! Responsibilities:
//! - Load configuration from environment (see `config.rs`)
//! - Initialize and run the WebSocket server (see `server.rs`)
//! - Print human-friendly startup diagnostics

mod config;
mod encryption;
mod server;
mod types;

/// Main entry point for the chat server.
///
/// This function:
/// 1. Initializes the Tokio runtime
/// 2. Loads environment variables from .env file (if present)
/// 3. Prints startup diagnostics
/// 4. Loads server configuration
/// 5. Starts the WebSocket server
///
/// # Returns
///
/// A Result indicating success or failure
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Load .env if present
            let _ = dotenvy::dotenv();

            println!("🚀 Starting Encrypted WebSocket Chat Server...");
            println!("🔐 Encryption: AES-256-GCM enabled");
            println!("🔑 Password Authentication: REQUIRED");

            let config = config::Config::from_env()?;
            println!("📍 Binding to address: {}", config.bind_addr);

            server::run_server(config).await?;
            println!("Backend server process finished.");
            Ok(())
        })
}
