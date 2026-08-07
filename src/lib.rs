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
pub mod key;
pub mod key_sequences;
pub mod logging;
pub mod model;
pub mod mouse;
pub mod nil_renderer;
pub mod options;
pub mod program;
pub mod renderer;
pub mod screen;
pub mod signals;
pub mod standard_renderer;
pub mod tty;

pub use commands::*;
pub use exec::*;
pub use focus::*;
pub use key::*;
pub use key_sequences::*;
pub use logging::*;
pub use model::*;
pub use mouse::*;
pub use nil_renderer::*;
pub use options::*;
pub use program::*;
pub use renderer::*;
pub use screen::*;
pub use signals::*;
pub use standard_renderer::*;
pub use tty::*;
