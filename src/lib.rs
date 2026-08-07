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
pub mod program;

pub use commands::*;
pub use key::*;
pub use model::*;
pub use mouse::*;
pub use program::*;
