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

/// BatchMsg is a message used to perform a bunch of commands concurrently with
/// no ordering guarantees. You can send a BatchMsg with `batch`.
pub struct BatchMsg(pub Vec<Cmd>);

impl fmt::Debug for BatchMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BatchMsg({} commands)", self.0.len())
    }
}

/// SequenceMsg is used internally to run the given commands in order.
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

/// Batch performs a bunch of commands concurrently with no ordering guarantees
/// about the results. Use `batch` to return several commands.
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

/// Sequence runs the given commands one at a time, in order. Contrast this with
/// `batch`, which runs commands concurrently.
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

/// Every is a command that ticks in sync with the system clock. So, if you
/// wanted to tick with the system clock every second, minute or hour you
/// could use this. It's also handy for having different things tick in sync.
///
/// Because we're ticking with the system clock the tick will likely not run for
/// the entire specified duration. For example, if we're ticking for one minute
/// and the clock is at 12:34:20 then the next tick will happen at 12:35:00, 40
/// seconds later.
///
/// To produce the command, pass a duration and a function which returns
/// a message containing the time at which the tick occurred.
///
/// **Beginners' note**: `every` sends a single message and won't automatically
/// dispatch messages at an interval. To do that, you'll want to return another
/// `every` command after receiving your tick message.
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

/// Tick produces a command at an interval independent of the system clock at
/// the given duration. That is, the timer begins precisely when invoked,
/// and runs for its entire duration.
///
/// To produce the command, pass a duration and a function which returns
/// a message containing the time at which the tick occurred.
///
/// **Beginners' note**: `tick` sends a single message and won't automatically
/// dispatch messages at an interval. To do that, you'll want to return another
/// `tick` command after receiving your tick message.
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

/// RequestWindowSize is a command that queries the terminal for its current
/// size. It delivers the results to `update` via a `WindowSizeMsg`. Keep in
/// mind that `WindowSizeMsg`s will automatically be delivered to `update` when
/// the Program starts and when the window dimensions change, so in many cases
/// you will not need to explicitly invoke this command.
pub fn request_window_size() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestWindowSizeMsg))))
}
