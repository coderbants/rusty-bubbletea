//! Cleanroom Rust port of upstream Go source file: `cursor.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Cursor Position & Shape
//!
//! Terminal cursor position structures, shape definitions, and position query requests.
//! </public-docs>

use crate::color::Color;
use crate::model::Cmd;

/// Position represents a position in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// X coordinate (column).
    pub x: usize,
    /// Y coordinate (row).
    pub y: usize,
}

/// CursorPositionMsg represents the terminal cursor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPositionMsg {
    /// X coordinate (column).
    pub x: usize,
    /// Y coordinate (row).
    pub y: usize,
}

/// CursorShape represents a terminal cursor shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    /// Block cursor shape.
    CursorBlock,
    /// Underline cursor shape.
    CursorUnderline,
    /// Bar cursor shape.
    CursorBar,
}

/// Cursor configuration for a View.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    /// Cursor position.
    pub position: Position,
    /// Cursor shape.
    pub shape: CursorShape,
    /// Whether cursor is blinking.
    pub blink: bool,
    /// Cursor color.
    pub color: Option<Color>,
}

impl Cursor {
    /// <upstream-comment>NewCursor returns a new cursor with the default settings and the given
    /// position.</upstream-comment>
    pub fn new(x: usize, y: usize) -> Cursor {
        Cursor {
            position: Position { x, y },
            shape: CursorShape::CursorBlock,
            blink: true,
            color: None,
        }
    }
}

/// RequestCursorPosMsg is a message that requests the cursor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestCursorPosMsg;

/// RequestCursorPosition is a command that requests the cursor position.
/// The cursor position will be sent as a [`CursorPositionMsg`] message.
pub fn request_cursor_position() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestCursorPosMsg))))
}
