//! Cleanroom Rust port of upstream Go source file: `paste.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Bracketed Paste Messages
//!
//! Paste messages (`PasteMsg`, `PasteStartMsg`, `PasteEndMsg`) emitted when receiving bracketed paste text.
//! </public-docs>

use std::fmt;

/// PasteMsg is a message that is emitted when a terminal receives pasted text
/// using bracketed-paste.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteMsg {
    /// Pasted string content.
    pub content: String,
}

impl fmt::Display for PasteMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// PasteStartMsg is a message that is emitted when the terminal starts the
/// bracketed-paste text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasteStartMsg;

/// PasteEndMsg is a message that is emitted when the terminal ends the
/// bracketed-paste text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasteEndMsg;
