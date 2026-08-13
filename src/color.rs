//! Cleanroom Rust port of upstream Go source file: `color.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Color Messages & Requests
//!
//! Color requests (`RequestBackgroundColor`, `RequestForegroundColor`,
//! `RequestCursorColor`) and response messages.
//!
//! The response messages wrap `rusty-ultraviolet`'s color events, which
//! mirror the upstream `uv.ForegroundColorEvent` / `BackgroundColorEvent` /
//! `CursorColorEvent` types (hex via OSC response parsing, darkness via the
//! HSL-based `is_dark_color`).
//! </public-docs>

use crate::model::Cmd;

/// The color value used by the color response messages.
///
/// This is an alias of the ultraviolet color event payload; use `to_hex` and
/// `is_dark` through the message wrappers.
pub type Color = rusty_x_ansi::color::RGBColor;

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
        self.0.hex()
    }

    /// Returns whether the color is dark.
    pub fn is_dark(&self) -> bool {
        is_dark_color(self.0)
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
        self.0.hex()
    }

    /// Returns whether the color is dark.
    pub fn is_dark(&self) -> bool {
        is_dark_color(self.0)
    }
}

/// CursorColorMsg represents a cursor color change message. This message is
/// emitted when the program requests the terminal cursor color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorColorMsg(pub Color);

impl CursorColorMsg {
    /// Returns the hex representation of the color.
    pub fn to_hex(&self) -> String {
        self.0.hex()
    }

    /// Returns whether the color is dark.
    pub fn is_dark(&self) -> bool {
        is_dark_color(self.0)
    }
}

/// is_dark_color returns whether the given color is dark, mirroring the
/// upstream `uv.isDarkColor` (HSL lightness < 0.5).
fn is_dark_color(c: rusty_x_ansi::color::RGBColor) -> bool {
    let (_, _, l) = rgb_to_hsl(c.r, c.g, c.b);
    l < 0.5
}

/// rgb_to_hsl converts an RGB triple to an HSL triple.
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rnot = f64::from(r) / 255.0;
    let gnot = f64::from(g) / 255.0;
    let bnot = f64::from(b) / 255.0;
    let (cmax, cmin) = get_max_min(rnot, gnot, bnot);
    let delta = cmax - cmin;
    let l = (cmax + cmin) / 2.0;
    let (h, s) = if delta == 0.0 {
        (0.0, 0.0)
    } else {
        let h = if cmax == rnot {
            60.0 * (((gnot - bnot) / delta).rem_euclid(6.0))
        } else if cmax == gnot {
            60.0 * (((bnot - rnot) / delta) + 2.0)
        } else {
            60.0 * (((rnot - gnot) / delta) + 4.0)
        };
        let h = if h < 0.0 { h + 360.0 } else { h };
        let s = delta / (1.0 - (2.0 * l - 1.0).abs());
        (h, s)
    };
    (h, round(s), round(l))
}

fn get_max_min(a: f64, b: f64, c: f64) -> (f64, f64) {
    let (ma, mi) = if a > b { (a, b) } else { (b, a) };
    if c > ma {
        (c, mi)
    } else if c < mi {
        (ma, c)
    } else {
        (ma, mi)
    }
}

fn round(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}
