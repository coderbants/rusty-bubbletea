//! Cleanroom Rust port of upstream Go source file: `screen.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Screen
//!
//! Screen buffer command constructors and screen state messages (`ClearScreen`, `HideCursor`, `ShowCursor`, `EnableMouseCellMotion`, `EnableMouseAllMotion`, `DisableMouse`, `EnableBracketedPaste`, `DisableBracketedPaste`, `EnableReportFocus`, `DisableReportFocus`).
//! </public-docs>

use crate::model::{Cmd, Msg};

/// ClearScreenMsg tells the program to clear the screen buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearScreenMsg;

/// ClearScreen returns a command to clear the screen.
pub fn clear_screen() -> Cmd {
    Some(Box::new(|| Some(Box::new(ClearScreenMsg))))
}

/// HideCursorMsg hides terminal cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HideCursorMsg;

/// HideCursor returns a command to hide terminal cursor.
pub fn hide_cursor() -> Cmd {
    Some(Box::new(|| Some(Box::new(HideCursorMsg))))
}

/// ShowCursorMsg shows terminal cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowCursorMsg;

/// ShowCursor returns a command to show terminal cursor.
pub fn show_cursor() -> Cmd {
    Some(Box::new(|| Some(Box::new(ShowCursorMsg))))
}

/// EnableMouseCellMotionMsg enables cell motion mouse mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnableMouseCellMotionMsg;

/// EnableMouseCellMotion returns a command to enable cell motion mouse mode.
pub fn enable_mouse_cell_motion() -> Cmd {
    Some(Box::new(|| Some(Box::new(EnableMouseCellMotionMsg))))
}

/// EnableMouseAllMotionMsg enables all motion mouse mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnableMouseAllMotionMsg;

/// EnableMouseAllMotion returns a command to enable all motion mouse mode.
pub fn enable_mouse_all_motion() -> Cmd {
    Some(Box::new(|| Some(Box::new(EnableMouseAllMotionMsg))))
}

/// DisableMouseMsg disables mouse mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisableMouseMsg;

/// DisableMouse returns a command to disable mouse mode.
pub fn disable_mouse() -> Cmd {
    Some(Box::new(|| Some(Box::new(DisableMouseMsg))))
}

/// EnableBracketedPasteMsg enables bracketed paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnableBracketedPasteMsg;

/// EnableBracketedPaste returns a command to enable bracketed paste.
pub fn enable_bracketed_paste() -> Cmd {
    Some(Box::new(|| Some(Box::new(EnableBracketedPasteMsg))))
}

/// DisableBracketedPasteMsg disables bracketed paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisableBracketedPasteMsg;

/// DisableBracketedPaste returns a command to disable bracketed paste.
pub fn disable_bracketed_paste() -> Cmd {
    Some(Box::new(|| Some(Box::new(DisableBracketedPasteMsg))))
}

/// EnableReportFocusMsg enables focus reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnableReportFocusMsg;

/// EnableReportFocus returns a command to enable focus reporting.
pub fn enable_report_focus() -> Cmd {
    Some(Box::new(|| Some(Box::new(EnableReportFocusMsg))))
}

/// DisableReportFocusMsg disables focus reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisableReportFocusMsg;

/// DisableReportFocus returns a command to disable focus reporting.
pub fn disable_report_focus() -> Cmd {
    Some(Box::new(|| Some(Box::new(DisableReportFocusMsg))))
}
