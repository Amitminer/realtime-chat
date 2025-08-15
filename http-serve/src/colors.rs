//! ANSI color codes for terminal output.
//!
//! This module provides constants for ANSI color codes that can be used
//! to add color to terminal output. These are used throughout the HTTP
//! server for better visual feedback.

/// Reset all formatting
pub const RESET: &str = "\x1b[0m";
/// Bold text
pub const BOLD: &str = "\x1b[1m";
/// Dim text
pub const DIM: &str = "\x1b[2m";

/// Red color
pub const RED: &str = "\x1b[31m";
/// Green color
pub const GREEN: &str = "\x1b[32m";
/// Yellow color
pub const YELLOW: &str = "\x1b[33m";
/// Blue color
pub const BLUE: &str = "\x1b[34m";
/// Magenta color
pub const MAGENTA: &str = "\x1b[35m";
/// Cyan color
pub const CYAN: &str = "\x1b[36m";

/// Bright cyan color
pub const BRIGHT_CYAN: &str = "\x1b[96m";
