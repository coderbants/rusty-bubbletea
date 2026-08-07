//! Cleanroom Rust port of upstream Go source file: `tea.go` (View definitions)
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

use crate::color::Color;
use crate::cursor::Cursor;
use crate::keyboard::KeyboardEnhancements;
use crate::model::Cmd;
use crate::mouse::MouseMsg;

/// MouseMode enum for declarative views.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseMode {
    /// Disable mouse events.
    MouseModeNone = 0,
    /// Cell motion mouse events.
    MouseModeCellMotion = 1,
    /// All motion mouse events.
    MouseModeAllMotion = 2,
}

impl Default for MouseMode {
    fn default() -> Self {
        MouseMode::MouseModeNone
    }
}

/// OnMouseFn closure type for View mouse handlers.
pub type OnMouseFn = Box<dyn Fn(MouseMsg) -> Cmd + Send + Sync>;

/// View represents a declarative terminal view in Bubble Tea v2.0.8.
pub struct View {
    /// Screen content formatted string.
    pub content: String,
    /// Optional mouse event interceptor.
    pub on_mouse: Option<OnMouseFn>,
    /// Optional cursor configuration.
    pub cursor: Option<Cursor>,
    /// Optional background color.
    pub background_color: Option<Color>,
    /// Optional foreground color.
    pub foreground_color: Option<Color>,
    /// Window title string.
    pub window_title: String,
    /// Alternate screen buffer toggle.
    pub alt_screen: bool,
    /// Focus reporting toggle.
    pub report_focus: bool,
    /// Disable bracketed paste mode toggle.
    pub disable_bracketed_paste_mode: bool,
    /// Mouse tracking mode.
    pub mouse_mode: MouseMode,
    /// Keyboard enhancements requested.
    pub keyboard_enhancements: KeyboardEnhancements,
}

impl Default for View {
    fn default() -> Self {
        Self {
            content: String::new(),
            on_mouse: None,
            cursor: None,
            background_color: None,
            foreground_color: None,
            window_title: String::new(),
            alt_screen: false,
            report_focus: false,
            disable_bracketed_paste_mode: false,
            mouse_mode: MouseMode::MouseModeNone,
            keyboard_enhancements: KeyboardEnhancements::default(),
        }
    }
}

impl View {
    /// Creates a new View with initial string content.
    pub fn new(content: &str) -> Self {
        let mut v = Self::default();
        v.set_content(content);
        v
    }

    /// Helper method to set view content.
    pub fn set_content(&mut self, s: &str) {
        self.content = s.to_string();
    }
}
