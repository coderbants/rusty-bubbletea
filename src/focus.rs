//! Cleanroom Rust port of upstream Go source file: `focus.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Focus & Blur Messages
//!
//! Focus state messages (`FocusMsg`, `BlurMsg`) emitted when the terminal gains or loses window focus.
//! </public-docs>

/// FocusMsg is emitted when terminal gains focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusMsg;

/// BlurMsg is emitted when terminal loses focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlurMsg;
