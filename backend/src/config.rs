//! Server configuration loader.
//!
//! This module handles loading configuration from environment variables
//! and applying sensible defaults. It also validates that required
//! configuration values are present and valid.
//!
//! Configuration variables:
//! - `BACKEND_HOST` (default `0.0.0.0`)
//! - `BACKEND_PORT` (default `9001`)
//! - `BIND_ADDR` (optional; overrides host/port)
//! - `SERVER_PASSWORD` (required)

use std::env;
use std::io::{Error, ErrorKind};

/// In-memory representation of server configuration.
///
/// This struct holds all the configuration values needed to run the server.
#[derive(Clone, Debug)]
pub struct Config {
    /// The address to bind to (e.g., "0.0.0.0:9001")
    pub bind_addr: String,
    /// The password required for client authentication
    pub server_password: String,
}

impl Config {
    /// Construct configuration from environment variables.
    ///
    /// This function reads configuration from environment variables,
    /// applies defaults where appropriate, and validates that required
    /// values are present and valid.
    ///
    /// # Returns
    ///
    /// A Result containing the Config if successful, or an error
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
