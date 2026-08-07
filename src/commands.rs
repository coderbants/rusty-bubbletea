//! Cleanroom Rust port of upstream Go source file: `commands.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Commands
//!
//! Standard built-in commands for controlling program behavior and terminal state.
//! </public-docs>

use crate::model::Cmd;
use std::fmt;

/// <upstream-comment>
/// BatchMsg is sent when multiple commands are batched.
/// </upstream-comment>
pub struct BatchMsg(pub Vec<Cmd>);

impl fmt::Debug for BatchMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BatchMsg({} commands)", self.0.len())
    }
}

/// <upstream-comment>
/// QuitMsg is sent when the program should terminate.
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuitMsg;

/// <upstream-comment>
/// Quit is a command that signals the program to exit.
/// </upstream-comment>
pub fn quit() -> Cmd {
    Some(Box::new(|| Some(Box::new(QuitMsg))))
}

/// <upstream-comment>
/// Batch combines multiple commands into a single command.
/// </upstream-comment>
pub fn batch(cmds: Vec<Cmd>) -> Cmd {
    let valid_cmds: Vec<_> = cmds.into_iter().collect();
    if valid_cmds.is_empty() {
        return None;
    }
    Some(Box::new(move || Some(Box::new(BatchMsg(valid_cmds)))))
}

/// <upstream-comment>
/// Sequence runs commands sequentially one after another.
/// </upstream-comment>
pub fn sequence(cmds: Vec<Cmd>) -> Cmd {
    batch(cmds)
}

/// <upstream-comment>
/// EnterAltScreenMsg is a message to enter the alternate screen buffer.
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnterAltScreenMsg;

/// <upstream-comment>
/// EnterAltScreen returns a command that enters the alternate screen buffer.
/// </upstream-comment>
pub fn enter_alt_screen() -> Cmd {
    Some(Box::new(|| Some(Box::new(EnterAltScreenMsg))))
}

/// <upstream-comment>
/// ExitAltScreenMsg is a message to exit the alternate screen buffer.
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitAltScreenMsg;

/// <upstream-comment>
/// ExitAltScreen returns a command that exits the alternate screen buffer.
/// </upstream-comment>
pub fn exit_alt_screen() -> Cmd {
    Some(Box::new(|| Some(Box::new(ExitAltScreenMsg))))
}

/// <upstream-comment>
/// WindowSizeMsg conveys window dimensions (width, height).
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSizeMsg {
    /// Width of terminal in columns.
    pub width: u16,
    /// Height of terminal in rows.
    pub height: u16,
}

impl WindowSizeMsg {
    /// Creates a new WindowSizeMsg.
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}
