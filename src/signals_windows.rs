//! Cleanroom Rust port of upstream Go source file: `signals_windows.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Signals (Windows)
//!
//! Windows console resize and signal handling abstractions.
//! </public-docs>

use crate::screen::WindowSizeMsg;
use crossterm::terminal::size as term_size;
use std::sync::mpsc::Sender;

/// Listens for window size resizes on Windows systems.
pub fn listen_for_resize_windows(tx: &Sender<Box<dyn crate::model::Msg>>) {
    if let Ok((w, h)) = term_size() {
        let _ = tx.send(Box::new(WindowSizeMsg {
            width: w as usize,
            height: h as usize,
        }));
    }
}
