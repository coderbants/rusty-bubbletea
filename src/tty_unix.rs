//! Cleanroom Rust port of upstream Go source file: `tty_unix.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # TTY (Unix)
//!
//! POSIX Unix TTY handle and raw mode initialization.
//! </public-docs>

/// Unix TTY initialization check.
pub fn is_unix_tty() -> bool {
    !cfg!(target_os = "windows")
}
