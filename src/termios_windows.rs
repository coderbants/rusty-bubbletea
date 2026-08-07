//! Cleanroom Rust port of upstream Go source file: `termios_windows.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Termios (Windows)
//!
//! Windows console mode termios helpers.
//! </public-docs>

/// Windows console termios indicator.
pub fn is_windows_termios() -> bool {
    cfg!(target_os = "windows")
}
