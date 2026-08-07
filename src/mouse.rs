//! Cleanroom Rust port of upstream Go source file: `mouse.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Mouse Messages & Events
//!
//! Mouse button definitions, `Mouse` struct, and typed mouse messages (`MouseClickMsg`, `MouseReleaseMsg`, `MouseWheelMsg`, `MouseMotionMsg`).
//! </public-docs>

use crate::key::KeyMod;
use std::fmt;

/// MouseButton enum matching X11 button codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// No button.
    MouseNone = 0,
    /// Left mouse button.
    MouseLeft = 1,
    /// Middle mouse button (scroll wheel press).
    MouseMiddle = 2,
    /// Right mouse button.
    MouseRight = 3,
    /// Scroll wheel up.
    MouseWheelUp = 4,
    /// Scroll wheel down.
    MouseWheelDown = 5,
    /// Scroll wheel left.
    MouseWheelLeft = 6,
    /// Scroll wheel right.
    MouseWheelRight = 7,
    /// Browser backward button.
    MouseBackward = 8,
    /// Browser forward button.
    MouseForward = 9,
    /// Button 10.
    MouseButton10 = 10,
    /// Button 11.
    MouseButton11 = 11,
}

/// Mouse event data struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mouse {
    /// Zero-based X coordinate (column).
    pub x: usize,
    /// Zero-based Y coordinate (row).
    pub y: usize,
    /// Button pressed/released.
    pub button: MouseButton,
    /// Key modifiers active.
    pub mod_keys: KeyMod,
}

impl fmt::Display for Mouse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({};{}) {:?}", self.x, self.y, self.button)
    }
}

/// MouseClickMsg represents a mouse button click message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseClickMsg(pub Mouse);

impl fmt::Display for MouseClickMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// MouseReleaseMsg represents a mouse button release message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseReleaseMsg(pub Mouse);

impl fmt::Display for MouseReleaseMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// MouseWheelMsg represents a mouse wheel message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseWheelMsg(pub Mouse);

impl fmt::Display for MouseWheelMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// MouseMotionMsg represents a mouse motion message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseMotionMsg(pub Mouse);

impl fmt::Display for MouseMotionMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.button != MouseButton::MouseNone {
            write!(f, "{}+motion", self.0)
        } else {
            write!(f, "{} motion", self.0)
        }
    }
}

/// MouseMsg enum abstraction covering click, release, wheel, and motion messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseMsg {
    /// Mouse click event.
    Click(MouseClickMsg),
    /// Mouse release event.
    Release(MouseReleaseMsg),
    /// Mouse wheel event.
    Wheel(MouseWheelMsg),
    /// Mouse motion event.
    Motion(MouseMotionMsg),
}

impl MouseMsg {
    /// Returns reference to underlying Mouse struct.
    pub fn mouse(&self) -> &Mouse {
        match self {
            MouseMsg::Click(m) => &m.0,
            MouseMsg::Release(m) => &m.0,
            MouseMsg::Wheel(m) => &m.0,
            MouseMsg::Motion(m) => &m.0,
        }
    }
}

impl fmt::Display for MouseMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.mouse())
    }
}
