//! Cleanroom Rust port of upstream Go source file: `commands.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Commands
//!
//! Built-in command functions (`batch`, `sequence`, `every`, `tick`, `request_window_size`).
//! </public-docs>

use crate::model::{Cmd, Msg};
use std::fmt;
use std::time::{Duration, SystemTime};

/// BatchMsg is sent when multiple commands are batched concurrently.
pub struct BatchMsg(pub Vec<Cmd>);

impl fmt::Debug for BatchMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BatchMsg({} commands)", self.0.len())
    }
}

/// SequenceMsg is used internally to run commands sequentially in order.
pub struct SequenceMsg(pub Vec<Cmd>);

impl fmt::Debug for SequenceMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SequenceMsg({} commands)", self.0.len())
    }
}

/// QuitMsg signals the program to exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuitMsg;

/// Quit is a command that signals the program to exit.
pub fn quit() -> Cmd {
    Some(Box::new(|| Some(Box::new(QuitMsg))))
}

/// Batch performs multiple commands concurrently with no ordering guarantees.
pub fn batch(cmds: Vec<Cmd>) -> Cmd {
    let mut valid_cmds = Vec::new();
    for cmd in cmds {
        if cmd.is_some() {
            valid_cmds.push(cmd);
        }
    }
    match valid_cmds.len() {
        0 => None,
        1 => valid_cmds.into_iter().next().unwrap(),
        _ => Some(Box::new(move || Some(Box::new(BatchMsg(valid_cmds))))),
    }
}

/// Sequence runs the given commands one at a time, in order.
pub fn sequence(cmds: Vec<Cmd>) -> Cmd {
    let mut valid_cmds = Vec::new();
    for cmd in cmds {
        if cmd.is_some() {
            valid_cmds.push(cmd);
        }
    }
    match valid_cmds.len() {
        0 => None,
        1 => valid_cmds.into_iter().next().unwrap(),
        _ => Some(Box::new(move || Some(Box::new(SequenceMsg(valid_cmds))))),
    }
}

/// Every is a command that ticks in sync with the system clock.
pub fn every<F>(duration: Duration, fn_msg: F) -> Cmd
where
    F: FnOnce(SystemTime) -> Option<Box<dyn Msg>> + Send + Sync + 'static,
{
    Some(Box::new(move || {
        let now = SystemTime::now();
        if let Ok(elapsed) = now.duration_since(SystemTime::UNIX_EPOCH) {
            let nanos = elapsed.as_nanos();
            let dur_nanos = duration.as_nanos();
            if dur_nanos > 0 {
                let rem = nanos % dur_nanos;
                let sleep_nanos = dur_nanos - rem;
                std::thread::sleep(Duration::from_nanos(sleep_nanos as u64));
            }
        } else {
            std::thread::sleep(duration);
        }
        fn_msg(SystemTime::now())
    }))
}

/// Tick produces a command at an interval independent of system clock.
pub fn tick<F>(duration: Duration, fn_msg: F) -> Cmd
where
    F: FnOnce(SystemTime) -> Option<Box<dyn Msg>> + Send + Sync + 'static,
{
    Some(Box::new(move || {
        std::thread::sleep(duration);
        fn_msg(SystemTime::now())
    }))
}

/// RequestWindowSizeMsg is a message that requests terminal window size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestWindowSizeMsg;

/// RequestWindowSize produces a command that queries terminal size.
pub fn request_window_size() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestWindowSizeMsg))))
}
