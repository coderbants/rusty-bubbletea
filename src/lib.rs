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
pub mod key;
pub mod model;
pub mod mouse;
pub mod nil_renderer;
pub mod program;
pub mod renderer;
pub mod standard_renderer;

pub use commands::*;
pub use key::*;
pub use model::*;
pub use mouse::*;
pub use nil_renderer::*;
pub use program::*;
pub use renderer::*;
pub use standard_renderer::*;
