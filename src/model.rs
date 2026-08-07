//! <public-docs>
//! # Model
//!
//! Model defines the core state interface for Elm-architecture applications in Bubble Tea.
//! </public-docs>

use std::any::Any;
use std::fmt::Debug;

/// <upstream-comment>
/// Msg represents an action or event in the Bubble Tea program.
/// </upstream-comment>
pub trait Msg: Any + Send + Sync + Debug {
    /// Helper to downcast Msg trait objects to concrete types.
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Send + Sync + Debug> Msg for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// <upstream-comment>
/// Cmd is an asynchronous operation that performs I/O and returns a Msg.
/// </upstream-comment>
pub type Cmd = Option<Box<dyn FnOnce() -> Option<Box<dyn Msg>> + Send + Sync + 'static>>;

/// <upstream-comment>
/// Model contains the program state and defines how to handle messages
/// and render the view.
/// </upstream-comment>
pub trait Model: Send + Sync + 'static {
    /// <upstream-comment>
    /// Init is the first command that will be executed when the program starts.
    /// </upstream-comment>
    fn init(&self) -> Cmd {
        None
    }

    /// <upstream-comment>
    /// Update is called when a message is received from the event loop.
    /// </upstream-comment>
    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd;

    /// <upstream-comment>
    /// View renders the program's UI as a string.
    /// </upstream-comment>
    fn view(&self) -> String;
}
