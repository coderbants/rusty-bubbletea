//! Cleanroom Rust port of upstream Go source file: `tea.go` (Model interface)
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

use crate::view::View;
use std::any::Any;
use std::fmt::Debug;

/// Msg represents an event message delivered to the Model's Update function.
pub trait Msg: Any + Send + Sync + Debug {
    /// Helper to downcast to Any.
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Send + Sync + Debug> Msg for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Cmd is an asynchronous command closure returning an optional Msg.
pub type Cmd = Option<Box<dyn FnOnce() -> Option<Box<dyn Msg>> + Send + Sync>>;

/// Model defines the application state machine according to The Elm Architecture in Bubble Tea v2.0.8.
pub trait Model: Send + Sync + Sized + 'static {
    /// Init is called when the program starts, returning an optional initial command.
    fn init(&self) -> Cmd {
        None
    }

    /// Update receives a message and returns an updated Model and optional command.
    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd;

    /// View renders the program's UI as a declarative `View` struct.
    fn view(&self) -> View;
}
