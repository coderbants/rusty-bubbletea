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
use std::time::{Duration, SystemTime};

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
/// SequenceMsg runs commands sequentially.
/// </upstream-comment>
pub struct SequenceMsg(pub Vec<Cmd>);

impl fmt::Debug for SequenceMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SequenceMsg({} commands)", self.0.len())
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
    let mut valid_cmds = Vec::new();
    for cmd in cmds {
        if cmd.is_some() {
            valid_cmds.push(cmd);
        }
    }
    if valid_cmds.is_empty() {
        return None;
    }
    Some(Box::new(move || Some(Box::new(BatchMsg(valid_cmds)))))
}

/// <upstream-comment>
/// Sequence runs the given commands one at a time, in order.
/// </upstream-comment>
pub fn sequence(cmds: Vec<Cmd>) -> Cmd {
    let mut valid_cmds = Vec::new();
    for cmd in cmds {
        if cmd.is_some() {
            valid_cmds.push(cmd);
        }
    }
    if valid_cmds.is_empty() {
        return None;
    }
    Some(Box::new(move || Some(Box::new(SequenceMsg(valid_cmds)))))
}

/// <upstream-comment>
/// Tick produces a command at an interval independent of system clock.
/// </upstream-comment>
pub fn tick<F>(duration: Duration, fn_msg: F) -> Cmd
where
    F: FnOnce(SystemTime) -> Option<Box<dyn crate::model::Msg>> + Send + Sync + 'static,
{
    Some(Box::new(move || {
        std::thread::sleep(duration);
        fn_msg(SystemTime::now())
    }))
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
/// SetWindowTitleMsg sets terminal window title.
/// </upstream-comment>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetWindowTitleMsg(pub String);

/// <upstream-comment>
/// SetWindowTitle produces a command that sets the terminal title.
/// </upstream-comment>
pub fn set_window_title(title: &str) -> Cmd {
    let t = title.to_string();
    Some(Box::new(move || Some(Box::new(SetWindowTitleMsg(t)))))
}

/// <upstream-comment>
/// RequestWindowSizeMsg signals a request to query window dimensions.
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestWindowSizeMsg;

/// <upstream-comment>
/// WindowSize produces a command to query current terminal window size.
/// </upstream-comment>
pub fn window_size() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestWindowSizeMsg))))
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
