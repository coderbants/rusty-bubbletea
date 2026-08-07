//! Cleanroom Rust port of upstream Go source file: `termcap.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Termcap / Terminfo Capabilities
//!
//! Terminal capability query requests (`request_capability`) and responses (`CapabilityMsg`).
//! </public-docs>

use crate::model::Cmd;
use std::fmt;

/// RequestCapabilityMsg is sent internally to query terminal capabilities (XTGETTCAP).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestCapabilityMsg(pub String);

/// RequestCapability produces a command that queries terminal capabilities.
pub fn request_capability(s: &str) -> Cmd {
    let cap = s.to_string();
    Some(Box::new(move || Some(Box::new(RequestCapabilityMsg(cap)))))
}

/// CapabilityMsg represents a Termcap/Terminfo response event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityMsg {
    /// Capability string content.
    pub content: String,
}

impl fmt::Display for CapabilityMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}
