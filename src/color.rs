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

    /// Calculates luminance to report if the color is dark.
    pub fn is_dark(&self) -> bool {
        let lum = 0.299 * (self.r as f64) + 0.587 * (self.g as f64) + 0.114 * (self.b as f64);
        lum < 128.0
    }
}

/// RequestBackgroundColorMsg requests terminal background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestBackgroundColorMsg;

/// RequestBackgroundColor produces a command that requests terminal background color.
pub fn request_background_color() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestBackgroundColorMsg))))
}

/// RequestForegroundColorMsg requests terminal foreground color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestForegroundColorMsg;

/// RequestForegroundColor produces a command that requests terminal foreground color.
pub fn request_foreground_color() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestForegroundColorMsg))))
}

/// RequestCursorColorMsg requests terminal cursor color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestCursorColorMsg;

/// RequestCursorColor produces a command that requests terminal cursor color.
pub fn request_cursor_color() -> Cmd {
    Some(Box::new(|| Some(Box::new(RequestCursorColorMsg))))
}

/// ForegroundColorMsg conveys terminal foreground color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForegroundColorMsg(pub Color);

impl ForegroundColorMsg {
    /// Returns hex string.
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }
    /// Returns whether color is dark.
    pub fn is_dark(&self) -> bool {
        self.0.is_dark()
    }
}

/// BackgroundColorMsg conveys terminal background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundColorMsg(pub Color);

impl BackgroundColorMsg {
    /// Returns hex string.
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }
    /// Returns whether color is dark.
    pub fn is_dark(&self) -> bool {
        self.0.is_dark()
    }
}

/// CursorColorMsg conveys terminal cursor color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorColorMsg(pub Color);

impl CursorColorMsg {
    /// Returns hex string.
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }
    /// Returns whether color is dark.
    pub fn is_dark(&self) -> bool {
        self.0.is_dark()
    }
}
