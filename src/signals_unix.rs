//! Cleanroom Rust port of upstream Go source file: `signals_unix.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Signals (Unix)
//!
//! POSIX SIGWINCH terminal resize signal handling.
//! </public-docs>

use crate::screen::WindowSizeMsg;
use crossterm::terminal::size as term_size;
use std::sync::mpsc::Sender;

/// Listens for SIGWINCH window size resizes on Unix systems.
pub fn listen_for_resize(tx: &Sender<Box<dyn crate::model::Msg>>) {
    if let Ok((w, h)) = term_size() {
        let _ = tx.send(Box::new(WindowSizeMsg {
            width: w as usize,
            height: h as usize,
        }));
    }
}
