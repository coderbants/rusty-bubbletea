//! Cleanroom Rust port of upstream Go source file: `tea.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <upstream-docs>
//! Package tea provides a framework for building rich terminal user interfaces
//! based on the paradigms of The Elm Architecture. It's well-suited for simple
//! and complex terminal applications, either inline, full-window, or a mix of
//! both. It's been battle-tested in several large projects and is
//! production-ready.
//!
//! A tutorial is available at https://github.com/charmbracelet/bubbletea/tree/master/tutorials
//!
//! Example programs can be found at https://github.com/charmbracelet/bubbletea/tree/master/examples
//! </upstream-docs>

#![deny(missing_docs)]

pub mod clipboard;
pub mod color;
pub mod commands;
pub mod cursed_renderer;
pub mod cursor;
pub mod environ;
pub mod exec;
pub mod focus;
pub mod input;
pub mod key;
pub mod keyboard;
pub mod logging;
pub mod mod_keys;
pub mod model;
pub mod mouse;
pub mod nil_renderer;
pub mod options;
pub mod paste;
pub mod profile;
pub mod program;
pub mod raw;
pub mod renderer;
pub mod screen;
pub mod signals_unix;
pub mod signals_windows;
pub mod termcap;
pub mod termios_bsd;
pub mod termios_other;
pub mod termios_unix;
pub mod termios_windows;
pub mod tty;
pub mod tty_unix;
pub mod tty_windows;
pub mod view;
pub mod xterm;

pub use clipboard::*;
pub use color::*;
pub use commands::*;
pub use cursed_renderer::*;
pub use cursor::*;
pub use environ::*;
pub use exec::*;
pub use focus::*;
pub use input::*;
pub use key::*;
pub use keyboard::*;
pub use logging::*;
pub use mod_keys::*;
pub use model::*;
pub use mouse::*;
pub use nil_renderer::*;
pub use options::*;
pub use paste::*;
pub use profile::*;
pub use program::*;
pub use raw::*;
pub use renderer::*;
pub use screen::*;
pub use termcap::*;
pub use tty::*;
pub use view::*;
pub use xterm::*;
