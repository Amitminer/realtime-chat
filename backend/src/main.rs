//! Encrypted WebSocket chat server entrypoint.
//!
//! Responsibilities:
//! - Load configuration from environment (see `config.rs`)
//! - Initialize and run the WebSocket server (see `server.rs`)
//! - Print human-friendly startup diagnostics
mod config;
mod encryption;
mod server;
mod types;

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

            server::run_server(config)
                .await
                .expect("Failed to Run server");
            println!("Backend server process finished.");
            Ok(())
        })
}
