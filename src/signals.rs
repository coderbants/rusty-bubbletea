//! Cleanroom Rust port of upstream Go source file: `signals_unix.go`, `signals_windows.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Signals
//!
//! Signal listeners and handling for SIGWINCH and SIGINT.
//! </public-docs>

use std::sync::mpsc::Sender;
use crate::commands::WindowSizeMsg;
use crossterm::terminal::size as term_size;

/// <upstream-comment>
/// Listens for window size resizes and sends WindowSizeMsg.
/// </upstream-comment>
pub fn check_resize(tx: &Sender<Box<dyn crate::model::Msg>>) {
    if let Ok((w, h)) = term_size() {
        let _ = tx.send(Box::new(WindowSizeMsg::new(w, h)));
    }
}
