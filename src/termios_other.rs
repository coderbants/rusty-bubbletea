//! Cleanroom Rust port of upstream Go source file: `termios_other.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Termios (Other)
//!
//! Non-POSIX termios fallback terminal configuration helpers.
//! </public-docs>

/// Non-POSIX termios fallback indicator.
pub fn is_other_termios() -> bool {
    !cfg!(unix) && !cfg!(target_os = "windows")
}
