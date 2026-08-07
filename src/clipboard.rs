//! Cleanroom Rust port of upstream Go source file: `clipboard.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Clipboard
//!
//! OSC52 terminal clipboard read/write command constructors (`SetClipboard`, `ReadClipboard`, `SetPrimaryClipboard`, `ReadPrimaryClipboard`) and `ClipboardMsg`.
//! </public-docs>

use crate::model::Cmd;
use std::fmt;

/// ClipboardMsg is emitted when receiving OSC52 clipboard data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardMsg {
    /// Content string.
    pub content: String,
    /// Selection byte ('c' system, 'p' primary).
    pub selection: u8,
}

impl ClipboardMsg {
    /// Returns selection type byte.
    pub fn clipboard(&self) -> u8 {
        self.selection
    }
}

impl fmt::Display for ClipboardMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// SetClipboardMsg requests setting system clipboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetClipboardMsg(pub String);

/// SetClipboard produces a command that sets system clipboard via OSC52.
pub fn set_clipboard(s: &str) -> Cmd {
    let text = s.to_string();
    Some(Box::new(move || Some(Box::new(SetClipboardMsg(text)))))
}

/// ReadClipboardMsg requests reading system clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadClipboardMsg;

/// ReadClipboard produces a command that reads system clipboard via OSC52.
pub fn read_clipboard() -> Cmd {
    Some(Box::new(|| Some(Box::new(ReadClipboardMsg))))
}

/// SetPrimaryClipboardMsg requests setting primary clipboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPrimaryClipboardMsg(pub String);

/// SetPrimaryClipboard produces a command that sets primary clipboard via OSC52.
pub fn set_primary_clipboard(s: &str) -> Cmd {
    let text = s.to_string();
    Some(Box::new(move || Some(Box::new(SetPrimaryClipboardMsg(text)))))
}

/// ReadPrimaryClipboardMsg requests reading primary clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadPrimaryClipboardMsg;

/// ReadPrimaryClipboard produces a command that reads primary clipboard via OSC52.
pub fn read_primary_clipboard() -> Cmd {
    Some(Box::new(|| Some(Box::new(ReadPrimaryClipboardMsg))))
}
