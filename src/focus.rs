//! Cleanroom Rust port of upstream Go source file: `focus.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Focus
//!
//! FocusMsg and BlurMsg representing terminal focus events.
//! </public-docs>

/// <upstream-comment>
/// FocusMsg represents a terminal focus message (gained focus).
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusMsg;

/// <upstream-comment>
/// BlurMsg represents a terminal blur message (lost focus).
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlurMsg;
