//! Cleanroom Rust port of upstream Go source file: `tty.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # TTY Terminal Management
//!
//! TTY initialization, input stream setup, raw mode toggles, and window dimension queries for Bubble Tea v2.0.8.
//! </public-docs>

use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size as term_size};

/// Initializes terminal raw mode.
pub fn init_terminal() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    Ok(())
}

/// Restores terminal raw mode.
pub fn restore_terminal() -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    Ok(())
}

/// Queries current window size columns and rows.
pub fn get_window_size() -> Result<(u16, u16), Box<dyn std::error::Error>> {
    let (w, h) = term_size()?;
    Ok((w, h))
}
