//! Cleanroom Rust port of upstream Go source file: `termios_unix.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Termios (Unix)
//!
//! POSIX Unix termios terminal configuration helpers.
//! </public-docs>

/// POSIX Unix termios indicator.
pub fn is_unix_termios() -> bool {
    cfg!(unix)
}
