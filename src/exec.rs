//! Cleanroom Rust port of upstream Go source file: `exec.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Process Execution
//!
//! External process execution commands (`exec_process`) for spawning sub-shells and editors in Bubble Tea v2.0.8.
//! </public-docs>

use crate::model::{Cmd, Msg};

/// Callback function type for ExecProcess.
pub type ExecCallback = fn(Result<(), std::io::Error>) -> Option<Box<dyn Msg>>;

/// ExecMsg is sent internally to trigger command execution.
#[derive(Debug)]
pub struct ExecMsg {
    /// Command name.
    pub cmd: String,
    /// Command arguments.
    pub args: Vec<String>,
}

/// ExecProcess spawns an external process (e.g. vim, htop) while pausing raw mode.
pub fn exec_process(cmd: &str, args: &[&str]) -> Cmd {
    let name = cmd.to_string();
    let argv = args.iter().map(|s| s.to_string()).collect();
    Some(Box::new(move || {
        Some(Box::new(ExecMsg {
            cmd: name,
            args: argv,
        }))
    }))
}
