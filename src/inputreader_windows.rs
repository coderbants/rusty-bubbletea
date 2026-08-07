//! Cleanroom Rust port of upstream Go source file: `inputreader_windows.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Input Reader (Windows)
//!
//! Windows VT console mode input reader abstractions.
//! </public-docs>

/// Returns whether Windows console mode input reader is active.
pub fn is_windows_input_reader() -> bool {
    cfg!(target_os = "windows")
}
