//! Cleanroom Rust port of upstream Go source file: `signals_unix.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Signals (Unix)
//!
//! POSIX SIGWINCH terminal resize signal handling.
//! </public-docs>

use std::sync::mpsc::Sender;
use crate::commands::WindowSizeMsg;
use crossterm::terminal::size as term_size;

/// Listens for SIGWINCH window size resizes on Unix systems.
pub fn listen_for_resize(tx: &Sender<Box<dyn crate::model::Msg>>) {
    if let Ok((w, h)) = term_size() {
        let _ = tx.send(Box::new(WindowSizeMsg::new(w, h)));
    }
}
