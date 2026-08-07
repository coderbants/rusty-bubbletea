//! Cleanroom Rust port of upstream Go source file: `inputreader_other.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Input Reader (Other / Non-Windows)
//!
//! Non-windows terminal input reader abstractions.
//! </public-docs>

/// Returns whether standard POSIX input reader is active.
pub fn is_posix_input_reader() -> bool {
    !cfg!(target_os = "windows")
}
