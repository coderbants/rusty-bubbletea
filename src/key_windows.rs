//! Cleanroom Rust port of upstream Go source file: `key_windows.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Key (Windows)
//!
//! Windows console API input reading abstractions.
//! </public-docs>

/// Windows key reading helper.
pub fn is_windows_key_reading() -> bool {
    cfg!(target_os = "windows")
}
