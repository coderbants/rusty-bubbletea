//! Cleanroom Rust port of upstream Go source file: `clipboard.go` (v2.0.0+)
//! Upstream Target Tag / Version: `v1.3.4` (forward-ported OSC52 clipboard features)
//!
//! <public-docs>
//! # Clipboard
//!
//! OSC52 terminal clipboard read/write command constructors (`SetClipboard`, `ReadClipboard`, `SetPrimaryClipboard`, `ReadPrimaryClipboard`) and `ClipboardMsg`.
//! </public-docs>

use crate::model::Cmd;
use std::fmt;

/// <upstream-comment>
/// ClipboardMsg is a clipboard read message event emitted when a terminal receives an OSC52 clipboard response.
/// </upstream-comment>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardMsg {
    /// Content read from the clipboard.
    pub content: String,
    /// Selection byte ('c' for system clipboard, 'p' for primary selection).
    pub selection: u8,
}

impl ClipboardMsg {
    /// Creates a new ClipboardMsg.
    pub fn new(content: &str, selection: u8) -> Self {
        Self {
            content: content.to_string(),
            selection,
        }
    }

    /// Returns the clipboard selection type byte.
    pub fn clipboard(&self) -> u8 {
        self.selection
    }
}

impl fmt::Display for ClipboardMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// SetClipboardMsg is sent to set system clipboard via OSC52.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetClipboardMsg(pub String);

/// SetClipboard produces a command that sets the system clipboard using OSC52.
pub fn set_clipboard(s: &str) -> Cmd {
    let content = s.to_string();
    Some(Box::new(move || Some(Box::new(SetClipboardMsg(content)))))
}

/// ReadClipboardMsg requests reading system clipboard via OSC52.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadClipboardMsg;

/// ReadClipboard produces a command that reads system clipboard via OSC52.
pub fn read_clipboard() -> Cmd {
    Some(Box::new(|| Some(Box::new(ReadClipboardMsg))))
}

/// SetPrimaryClipboardMsg is sent to set primary selection clipboard via OSC52.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPrimaryClipboardMsg(pub String);

/// SetPrimaryClipboard produces a command that sets primary clipboard using OSC52.
pub fn set_primary_clipboard(s: &str) -> Cmd {
    let content = s.to_string();
    Some(Box::new(move || Some(Box::new(SetPrimaryClipboardMsg(content)))))
}

/// ReadPrimaryClipboardMsg requests reading primary selection clipboard via OSC52.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadPrimaryClipboardMsg;

/// ReadPrimaryClipboard produces a command that reads primary clipboard using OSC52.
pub fn read_primary_clipboard() -> Cmd {
    Some(Box::new(|| Some(Box::new(ReadPrimaryClipboardMsg))))
}
