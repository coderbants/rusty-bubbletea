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

/// ClipboardMsg is a clipboard read message event. This message is emitted when
/// a terminal receives an OSC52 clipboard read message event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardMsg {
    /// Content string.
    pub content: String,
    /// Selection byte ('c' system, 'p' primary).
    pub selection: u8,
}

impl ClipboardMsg {
    /// Returns the clipboard selection type. This will be one of the
    /// following values:
    ///
    /// - `c`: System clipboard.
    /// - `p`: Primary clipboard (X11/Wayland only).
    pub fn clipboard(&self) -> u8 {
        self.selection
    }
}

impl fmt::Display for ClipboardMsg {
    /// Returns the string representation of the clipboard message.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// SetClipboardMsg is an internal message used to set the system clipboard
/// using OSC52.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetClipboardMsg(pub String);

/// SetClipboard produces a command that sets the system clipboard using OSC52.
/// Note that OSC52 is not supported in all terminals.
pub fn set_clipboard(s: &str) -> Cmd {
    let text = s.to_string();
    Some(Box::new(move || Some(Box::new(SetClipboardMsg(text)))))
}

/// ReadClipboardMsg is an internal message used to read the system clipboard
/// using OSC52.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadClipboardMsg;

/// ReadClipboard produces a command that reads the system clipboard using OSC52.
/// Note that OSC52 is not supported in all terminals.
pub fn read_clipboard() -> Cmd {
    Some(Box::new(|| Some(Box::new(ReadClipboardMsg))))
}

/// SetPrimaryClipboardMsg is an internal message used to set the primary
/// clipboard using OSC52.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPrimaryClipboardMsg(pub String);

/// SetPrimaryClipboard produces a command that sets the primary clipboard using
/// OSC52. Primary clipboard selection is a feature present in X11 and Wayland
/// only.
/// Note that OSC52 is not supported in all terminals.
pub fn set_primary_clipboard(s: &str) -> Cmd {
    let text = s.to_string();
    Some(Box::new(move || Some(Box::new(SetPrimaryClipboardMsg(text)))))
}

/// ReadPrimaryClipboardMsg is an internal message used to read the primary
/// clipboard using OSC52.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadPrimaryClipboardMsg;

/// ReadPrimaryClipboard produces a command that reads the primary clipboard
/// using OSC52. Primary clipboard selection is a feature present in X11 and
/// Wayland only.
/// Note that OSC52 is not supported in all terminals.
pub fn read_primary_clipboard() -> Cmd {
    Some(Box::new(|| Some(Box::new(ReadPrimaryClipboardMsg))))
}
