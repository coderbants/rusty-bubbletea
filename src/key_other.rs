//! Cleanroom Rust port of upstream Go source file: `key_other.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Key (Other / Non-Windows)
//!
//! POSIX non-windows terminal key reading abstractions.
//! </public-docs>

/// Non-windows key reading helper.
pub fn is_windows_key_reading() -> bool {
    false
}
