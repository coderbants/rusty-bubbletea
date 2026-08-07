//! Cleanroom Rust port of upstream Go source file: `screen.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Screen Buffer & Window Size
//!
//! `WindowSizeMsg`, `clear_screen`, `ClearScreenMsg`, `ModeReportMsg`.
//! </public-docs>

use crate::model::Cmd;

/// WindowSizeMsg reports terminal size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSizeMsg {
    /// Width of terminal in columns.
    pub width: usize,
    /// Height of terminal in rows.
    pub height: usize,
}

/// ClearScreenMsg signals clearing the terminal screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearScreenMsg;

/// ClearScreen returns a command to clear the screen.
pub fn clear_screen() -> Cmd {
    Some(Box::new(|| Some(Box::new(ClearScreenMsg))))
}

/// ModeReportMsg represents a DECRPM mode report event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeReportMsg {
    /// Mode number.
    pub mode: u32,
    /// Mode setting value.
    pub value: u32,
}
