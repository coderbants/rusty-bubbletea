//! Cleanroom Rust port of upstream Go source file: `tty.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <user-docs>
//! # TTY Terminal Management
//!
//! TTY initialization, raw mode toggles, and window dimension queries for
//! Bubble Tea v2.0.8. Unix targets use the upstream-compatible termios and
//! raw-file-descriptor implementation. Windows targets use crossterm's safe
//! console-mode API for enable, disable, initialize, and restore operations.
//! </user-docs>

use crossterm::terminal::size as term_size;

#[cfg(unix)]
use crate::tty_unix as platform;
#[cfg(windows)]
use crate::tty_windows as platform;
#[cfg(not(any(unix, windows)))]
use unsupported as platform;

#[cfg(not(any(unix, windows)))]
mod unsupported {
    /// Reports that raw mode is unavailable on an unsupported target.
    pub(super) fn enable_raw_mode() -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "raw terminal mode is unsupported on this target",
        ))
    }

    /// Reports that raw mode is unavailable on an unsupported target.
    pub(super) fn disable_raw_mode() -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "raw terminal mode is unsupported on this target",
        ))
    }
}

/// Enables terminal raw mode through the target platform implementation.
pub fn enable_raw_mode() -> std::io::Result<()> {
    platform::enable_raw_mode()
}

/// Disables terminal raw mode through the target platform implementation.
pub fn disable_raw_mode() -> std::io::Result<()> {
    platform::disable_raw_mode()
}

/// Initializes terminal raw mode through the target platform implementation.
pub fn init_terminal() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    Ok(())
}

/// Restores terminal raw mode through the target platform implementation.
pub fn restore_terminal() -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    Ok(())
}

/// Queries current window size columns and rows.
pub fn get_window_size() -> Result<(u16, u16), Box<dyn std::error::Error>> {
    let (w, h) = term_size()?;
    Ok((w, h))
}
