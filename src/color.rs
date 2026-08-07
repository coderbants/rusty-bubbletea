//! Cleanroom Rust port of upstream Go source file: `color.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Color Messages & Requests
//!
//! Color requests (`RequestBackgroundColor`, `RequestForegroundColor`, `RequestCursorColor`) and response messages.
//! </public-docs>

use crate::model::Cmd;

/// RGBA Color structure matching Go's image/color.Color interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
    /// Alpha component (0-255).
    pub a: u8,
}

impl Color {
    /// Creates a new Color.
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Returns hex representation string.
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Returns whether the color is dark.
    pub fn is_dark(&self) -> bool {
        let lum = 0.299 * (self.r as f64) + 0.587 * (self.g as f64) + 0.114 * (self.b as f64);
        lum < 128.0
    }
}

/// RequestBackgroundColor is a command that requests the terminal background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestBackgroundColorMsg;

/// RequestBackgroundColor is a command that requests the terminal background color.
pub fn request_background_color() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestBackgroundColorMsg))))
}

/// RequestForegroundColorMsg is a message that requests the terminal foreground color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestForegroundColorMsg;

/// RequestForegroundColor is a command that requests the terminal foreground color.
pub fn request_foreground_color() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestForegroundColorMsg))))
}

/// RequestCursorColorMsg is a message that requests the terminal cursor color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestCursorColorMsg;

/// RequestCursorColor is a command that requests the terminal cursor color.
pub fn request_cursor_color() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestCursorColorMsg))))
}

/// ForegroundColorMsg represents a foreground color message. This message is
/// emitted when the program requests the terminal foreground color with the
/// `request_foreground_color` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForegroundColorMsg(pub Color);

impl ForegroundColorMsg {
    /// Returns the hex representation of the color.
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Returns whether the color is dark.
    pub fn is_dark(&self) -> bool {
        self.0.is_dark()
    }
}

/// BackgroundColorMsg represents a background color message. This message is
/// emitted when the program requests the terminal background color with the
/// `request_background_color` command.
///
/// This is commonly used in `Model::init` to get the terminal background color
/// for style definitions. For that you'll want to call `is_dark()` to determine
/// if the color is dark or light. For example:
///
/// ```rust,ignore
/// fn init(&self) -> Cmd { request_background_color() }
///
/// fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
///     if let Some(bg) = msg.as_any().downcast_ref::<BackgroundColorMsg>() {
///         self.styles = new_styles(bg.is_dark());
///     }
///     None
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundColorMsg(pub Color);

impl BackgroundColorMsg {
    /// Returns the hex representation of the color.
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Returns whether the color is dark.
    pub fn is_dark(&self) -> bool {
        self.0.is_dark()
    }
}

/// CursorColorMsg represents a cursor color change message. This message is
/// emitted when the program requests the terminal cursor color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorColorMsg(pub Color);

impl CursorColorMsg {
    /// Returns the hex representation of the color.
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Returns whether the color is dark.
    pub fn is_dark(&self) -> bool {
        self.0.is_dark()
    }
}
