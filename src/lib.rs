//! Cleanroom Rust port of upstream Go source file: `tea.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Charming Bubble Tea
//!
//! Cleanroom Rust port of Charmbracelet's Bubble Tea TUI Elm-architecture framework.
//! </public-docs>

#![deny(missing_docs)]

pub mod commands;
pub mod exec;
pub mod focus;
pub mod inputreader_other;
pub mod inputreader_windows;
pub mod key;
pub mod key_other;
pub mod key_sequences;
pub mod key_windows;
pub mod logging;
pub mod model;
pub mod mouse;
pub mod nil_renderer;
pub mod options;
pub mod program;
pub mod renderer;
pub mod screen;
pub mod signals_unix;
pub mod signals_windows;
pub mod standard_renderer;
pub mod tea_init;
pub mod tty;
pub mod tty_unix;
pub mod tty_windows;

pub use commands::*;
pub use exec::*;
pub use focus::*;
pub use inputreader_other::*;
pub use inputreader_windows::*;
pub use key::*;
pub use key_other::*;
pub use key_sequences::*;
pub use key_windows::*;
pub use logging::*;
pub use model::*;
pub use mouse::*;
pub use nil_renderer::*;
pub use options::*;
pub use program::*;
pub use renderer::*;
pub use screen::*;
pub use signals_unix::*;
pub use signals_windows::*;
pub use standard_renderer::*;
pub use tea_init::*;
pub use tty::*;
pub use tty_unix::*;
pub use tty_windows::*;
