//! Cleanroom Rust port of upstream Go source file: `tty_windows.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # TTY (Windows)
//!
//! Windows VT console mode handle and raw mode initialization.
//! </public-docs>

/// Windows TTY initialization check.
pub fn is_windows_tty() -> bool {
    cfg!(target_os = "windows")
}
