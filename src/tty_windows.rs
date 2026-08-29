//! Cleanroom Rust port of upstream Go source file: `tty_windows.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <user-docs>
//! # TTY (Windows)
//!
//! Windows VT console mode handle and raw mode initialization. Windows uses
//! crossterm's safe console-mode operations; no Unix termios or handwritten
//! raw-file-descriptor code is compiled for this target.
//! </user-docs>

/// Windows TTY initialization check.
pub fn is_windows_tty() -> bool {
    cfg!(windows)
}

#[cfg(windows)]
/// Enables Windows console raw mode through crossterm's safe API.
pub(crate) fn enable_raw_mode() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()
}

#[cfg(windows)]
/// Disables Windows console raw mode through crossterm's safe API.
pub(crate) fn disable_raw_mode() -> std::io::Result<()> {
    crossterm::terminal::disable_raw_mode()
}
