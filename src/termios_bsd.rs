//! Cleanroom Rust port of upstream Go source file: `termios_bsd.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Termios (BSD)
//!
//! BSD termios terminal configuration helpers.
//! </public-docs>

/// BSD termios support indicator.
pub fn is_bsd_termios() -> bool {
    cfg!(target_os = "freebsd") || cfg!(target_os = "macos") || cfg!(target_os = "openbsd") || cfg!(target_os = "netbsd")
}
