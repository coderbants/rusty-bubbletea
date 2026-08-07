//! Cleanroom Rust port of upstream Go source file: `screen.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Screen Buffer & Window Size
//!
//! `WindowSizeMsg`, `clear_screen`, `ClearScreenMsg`, `ModeReportMsg`.
//! </public-docs>

use crate::model::Cmd;

/// WindowSizeMsg is used to report the terminal size. It's sent to `update`
/// once initially and then on every terminal resize. Note that Windows does not
/// have support for reporting when resizes occur as it does not support the
/// SIGWINCH signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSizeMsg {
    /// Width of terminal in columns.
    pub width: usize,
    /// Height of terminal in rows.
    pub height: usize,
}

/// ClearScreenMsg is an internal message that signals to clear the screen.
/// You can send a ClearScreenMsg with `clear_screen`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearScreenMsg;

/// ClearScreen is a special command that tells the program to clear the screen
/// before the next update. This can be used to move the cursor to the top left
/// of the screen and clear visual clutter when the alt screen is not in use.
///
/// Note that it should never be necessary to call `clear_screen` for regular
/// redraws.
pub fn clear_screen() -> Cmd {
    Some(Box::new(|| Some(Box::new(ClearScreenMsg))))
}

/// ModeReportMsg is a message that represents a mode report event (DECRPM).
///
/// This is sent by the terminal in response to a request for a terminal mode
/// report (DECRQM). It indicates the current setting of a specific terminal
/// mode like cursor visibility, mouse tracking, etc.
///
/// See: <https://vt100.net/docs/vt510-rm/DECRPM.html>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeReportMsg {
    /// Mode is the mode number.
    pub mode: u32,
    /// Value is the mode setting value.
    pub value: u32,
}
