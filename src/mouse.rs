//! Cleanroom Rust port of upstream Go source file: `mouse.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Mouse
//!
//! Mouse represents mouse button clicks, motion events, and coordinates.
//! </public-docs>

use std::fmt;

/// <upstream-comment>
/// MouseButton indicates which mouse button was triggered.
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// <upstream-comment>
    /// MouseLeft button.
    /// </upstream-comment>
    MouseLeft,
    /// <upstream-comment>
    /// MouseRight button.
    /// </upstream-comment>
    MouseRight,
    /// <upstream-comment>
    /// MouseMiddle button.
    /// </upstream-comment>
    MouseMiddle,
    /// <upstream-comment>
    /// MouseWheelUp button.
    /// </upstream-comment>
    MouseWheelUp,
    /// <upstream-comment>
    /// MouseWheelDown button.
    /// </upstream-comment>
    MouseWheelDown,
    /// <upstream-comment>
    /// MouseRelease button event.
    /// </upstream-comment>
    MouseRelease,
    /// <upstream-comment>
    /// MouseUnknown event button.
    /// </upstream-comment>
    MouseUnknown,
}

/// <upstream-comment>
/// MouseAction indicates the type of mouse event (press, release, motion).
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseAction {
    /// <upstream-comment>
    /// MouseActionPress represents a button press.
    /// </upstream-comment>
    MouseActionPress,
    /// <upstream-comment>
    /// MouseActionRelease represents a button release.
    /// </upstream-comment>
    MouseActionRelease,
    /// <upstream-comment>
    /// MouseActionMotion represents mouse movement.
    /// </upstream-comment>
    MouseActionMotion,
}

/// <upstream-comment>
/// MouseMsg represents a mouse event with coordinates and buttons.
/// </upstream-comment>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseMsg {
    /// Column X coordinate (0-indexed).
    pub x: u16,
    /// Row Y coordinate (0-indexed).
    pub y: u16,
    /// Button associated with the event.
    pub button: MouseButton,
    /// Action associated with the event.
    pub action: MouseAction,
    /// Alt modifier key status.
    pub alt: bool,
    /// Ctrl modifier key status.
    pub ctrl: bool,
    /// Shift modifier key status.
    pub shift: bool,
}

impl MouseMsg {
    /// Creates a new MouseMsg.
    pub fn new(x: u16, y: u16, button: MouseButton, action: MouseAction) -> Self {
        Self {
            x,
            y,
            button,
            action,
            alt: false,
            ctrl: false,
            shift: false,
        }
    }
}

impl fmt::Display for MouseMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}: {:?} {:?})",
            self.x, self.y, self.button, self.action
        )
    }
}
