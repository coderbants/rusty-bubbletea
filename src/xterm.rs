//! Cleanroom Rust port of upstream Go source file: `xterm.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # XTerm Commands & Messages
//!
//! Terminal version queries (`request_terminal_version`) and responses (`TerminalVersionMsg`).
//! </public-docs>

use crate::model::Cmd;
use std::fmt;

/// TerminalVersionMsg represents the terminal version (XTVERSION).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalVersionMsg {
    /// Terminal version string name.
    pub name: String,
}

impl fmt::Display for TerminalVersionMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// RequestTerminalVersionMsg is sent internally to request terminal version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestTerminalVersionMsg;

/// RequestTerminalVersion produces a command that queries terminal version using XTVERSION.
pub fn request_terminal_version() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestTerminalVersionMsg))))
}
