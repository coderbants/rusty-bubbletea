//! Cleanroom Rust port of upstream Go source file: `renderer.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Renderer Trait
//!
//! Renderer interface trait for Bubble Tea v2.0.8 (`render(View)`, `flush(bool)`, `insert_above`, `clear_screen`).
//! </public-docs>

use crate::model::Cmd;
use crate::mouse::MouseMsg;
use crate::view::View;
use std::fmt;

/// Renderer interface for Bubble Tea v2.0.8.
pub trait Renderer: Send + Sync {
    /// Starts the renderer.
    fn start(&mut self);

    /// Closes the renderer and flushes any remaining data.
    fn close(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Renders a declarative View frame.
    fn render(&mut self, view: View);

    /// Flushes renderer output buffer to terminal stdout.
    fn flush(&mut self, closing: bool) -> Result<(), Box<dyn std::error::Error>>;

    /// Resets renderer to initial state.
    fn reset(&mut self);

    /// Inserts unmanaged lines above the TUI renderer.
    fn insert_above(&mut self, s: String) -> Result<(), Box<dyn std::error::Error>>;

    /// Resize notification.
    fn resize(&mut self, width: usize, height: usize);

    /// Clears terminal screen.
    fn clear_screen(&mut self);

    /// Write raw string to output.
    fn write_string(&mut self, s: &str) -> Result<usize, Box<dyn std::error::Error>>;

    /// Mouse event interceptor.
    fn on_mouse(&mut self, msg: MouseMsg) -> Cmd;

    /// Sets the cursor movement optimizations (hard tabs, backspace,
    /// newline mapping).
    fn set_optimizations(&mut self, hard_tabs: bool, backspace: bool, map_nl: bool);

    /// Sets the terminal color profile used for downsampling colors.
    fn set_color_profile(&mut self, p: charming_colorprofile::Profile);
}

/// PrintLineMsg represents a line printed above the TUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintLineMsg {
    /// Message body.
    pub message_body: String,
}

/// Println prints above the Program.
pub fn print_ln(args: fmt::Arguments<'_>) -> Cmd {
    let body = args.to_string();
    Some(Box::new(move || {
        Some(Box::new(PrintLineMsg { message_body: body }))
    }))
}

/// Printf prints formatted output above the Program.
pub fn print_f(args: fmt::Arguments<'_>) -> Cmd {
    let body = args.to_string();
    Some(Box::new(move || {
        Some(Box::new(PrintLineMsg { message_body: body }))
    }))
}
