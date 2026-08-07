//! Cleanroom Rust port of upstream Go source file: `raw.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Raw Command Output
//!
//! `raw` command sending raw escape sequences or queries to the terminal.
//! </public-docs>

use crate::model::Cmd;

/// RawMsg contains raw data to send to terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMsg(pub String);

/// Raw produces a command that prints the given string directly to stdout.
pub fn raw(s: &str) -> Cmd {
    let payload = s.to_string();
    Some(Box::new(move || Some(Box::new(RawMsg(payload)))))
}
