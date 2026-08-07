//! Cleanroom Rust port of upstream Go source file: `exec.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Exec
//!
//! ExecProcess and ExecCommand for executing external process commands with terminal release/restore.
//! </public-docs>

use crate::model::{Cmd, Msg};
use std::process::Command;

/// <upstream-comment>
/// ExecMsg is sent to run an external process command.
/// </upstream-comment>
pub struct ExecMsg {
    /// Command runner closure.
    pub cmd_fn: Box<dyn FnOnce() -> Option<Box<dyn Msg>> + Send + Sync>,
}

/// <upstream-comment>
/// ExecProcess runs an external command in a blocking fashion, pausing the program event loop.
/// </upstream-comment>
pub fn exec_process<F>(mut command: Command, callback: F) -> Cmd
where
    F: FnOnce(Result<(), std::io::Error>) -> Option<Box<dyn Msg>> + Send + Sync + 'static,
{
    Some(Box::new(move || {
        let status = command.status();
        let result = match status {
            Ok(s) if s.success() => Ok(()),
            Ok(s) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Process exited with status {}", s),
            )),
            Err(e) => Err(e),
        };
        callback(result)
    }))
}
