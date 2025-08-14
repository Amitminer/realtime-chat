//! Server configuration loader.
//! Reads values from environment variables and applies sensible defaults.
//!
//! Variables:
//! - `HOST` (default `0.0.0.0`)
//! - `PORT` (default `9001`)
//! - `BIND_ADDR` (optional; overrides host/port)
//! - `SERVER_PASSWORD` (required)
use std::env;

/// In-memory representation of server configuration.
#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub server_password: String,
}

impl Config {
    /// Construct configuration from environment variables.
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port: u16 = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(9001);
        let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| format!("{host}:{port}"));

        let server_password = env::var("SERVER_PASSWORD")
            .map_err(|_| "SERVER_PASSWORD is required. Set it in environment or .env")?;

        Ok(Self { bind_addr, server_password })
    }
}
