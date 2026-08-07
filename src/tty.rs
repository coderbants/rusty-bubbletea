//! Cleanroom Rust port of upstream Go source file: `tty.go`, `tty_unix.go`, `tty_windows.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # TTY
//!
//! Terminal initialization and state restoration helpers.
//! </public-docs>

use crossterm::{
    cursor::Show,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use std::io::{stdout, Result};

/// <upstream-comment>
/// Enable raw mode and prepare TTY.
/// </upstream-comment>
pub fn init_terminal() -> Result<()> {
    enable_raw_mode()
}

/// <upstream-comment>
/// Restore original TTY terminal state.
/// </upstream-comment>
pub fn restore_terminal() -> Result<()> {
    let _ = execute!(stdout(), crossterm::event::DisableMouseCapture);
    let _ = execute!(stdout(), Show, LeaveAlternateScreen);
    disable_raw_mode()
}
