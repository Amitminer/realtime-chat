//! Server configuration loader.
//! Reads values from environment variables and applies sensible defaults.
//!
//! Variables:
//! - `HOST` (default `0.0.0.0`)
//! - `PORT` (default `9001`)
//! - `BIND_ADDR` (optional; overrides host/port)
//! - `SERVER_PASSWORD` (required)
use std::env;
use std::io::{Error, ErrorKind};

/// In-memory representation of server configuration.
#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub server_password: String,
}

impl Config {
    /// Construct configuration from environment variables.
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Coalesce unset or empty HOST to default
        // Coalesce unset or empty HOST to default
        let host = env::var("BACKEND_HOST")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "0.0.0.0".to_string());

        // Coalesce unset or unparsable PORT to default
        let port: u16 = env::var("BACKEND_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(9001);

        // If BIND_ADDR is provided but empty, ignore it and build from host/port
        let bind_addr = env::var("BIND_ADDR")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| format!("{host}:{port}"));

        // SERVER_PASSWORD must be present and non-empty
        let server_password = env::var("SERVER_PASSWORD")
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        if server_password.is_empty() {
            return Err(Box::new(Error::new(
                ErrorKind::InvalidInput,
                "SERVER_PASSWORD cannot be empty",
            )));
        }

        Ok(Self { bind_addr, server_password })
    }
}
