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
use std::sync::Arc;

/// MouseMode enum for declarative views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseMode {
    /// Disable mouse events.
    #[default]
    MouseModeNone = 0,
    /// Cell motion mouse events.
    MouseModeCellMotion = 1,
    /// All motion mouse events.
    MouseModeAllMotion = 2,
}

/// ProgressBarState represents the state of the progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressBarState {
    /// No progress bar.
    ProgressBarNone = 0,
    /// Default progress bar state.
    ProgressBarDefault,
    /// Error progress bar state.
    ProgressBarError,
    /// Indeterminate progress bar state.
    ProgressBarIndeterminate,
    /// Warning progress bar state.
    ProgressBarWarning,
}

impl ProgressBarState {
    /// Returns a human-readable value for the given state.
    pub fn to_string(&self) -> &'static str {
        match self {
            ProgressBarState::ProgressBarNone => "None",
            ProgressBarState::ProgressBarDefault => "Default",
            ProgressBarState::ProgressBarError => "Error",
            ProgressBarState::ProgressBarIndeterminate => "Indeterminate",
            ProgressBarState::ProgressBarWarning => "Warning",
        }
    }
}

/// ProgressBar represents the terminal progress bar.
///
/// Support depends on the terminal.
///
/// See <https://learn.microsoft.com/en-us/windows/terminal/tutorials/progress-bar-sequences>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressBar {
    /// State is the current state of the progress bar. It can be one of
    /// [ProgressBarState::ProgressBarNone], [ProgressBarState::ProgressBarDefault],
    /// [ProgressBarState::ProgressBarError], [ProgressBarState::ProgressBarIndeterminate],
    /// and [ProgressBarState::ProgressBarWarning].
    pub state: ProgressBarState,
    /// Value is the current value of the progress bar. It should be between
    /// 0 and 100.
    pub value: usize,
}

/// NewProgressBar returns a new progress bar with the given state and value.
/// The value is ignored if the state is [ProgressBarState::ProgressBarNone] or
/// [ProgressBarState::ProgressBarIndeterminate].
pub fn new_progress_bar(state: ProgressBarState, value: usize) -> ProgressBar {
    ProgressBar {
        state,
        value: value.clamp(0, 100),
    }
}

/// OnMouseFn closure type for View mouse handlers.
///
/// `Arc` (rather than `Box`) so that [View] clones preserve the handler:
/// the renderer clones the view each frame and needs the closure on
/// `last_view` to route mouse messages back to the model.
pub type OnMouseFn = Arc<dyn Fn(MouseMsg) -> Cmd + Send + Sync>;

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
    /// Optional terminal progress bar.
    pub progress_bar: Option<ProgressBar>,
}

impl Clone for View {
    fn clone(&self) -> View {
        View {
            content: self.content.clone(),
            on_mouse: self.on_mouse.clone(),
            cursor: self.cursor.clone(),
            background_color: self.background_color,
            foreground_color: self.foreground_color,
            window_title: self.window_title.clone(),
            alt_screen: self.alt_screen,
            report_focus: self.report_focus,
            disable_bracketed_paste_mode: self.disable_bracketed_paste_mode,
            mouse_mode: self.mouse_mode,
            keyboard_enhancements: self.keyboard_enhancements,
            progress_bar: self.progress_bar,
        }
    }
}

impl PartialEq for View {
    fn eq(&self, other: &View) -> bool {
        self.content == other.content
            && self.cursor == other.cursor
            && self.background_color == other.background_color
            && self.foreground_color == other.foreground_color
            && self.window_title == other.window_title
            && self.alt_screen == other.alt_screen
            && self.report_focus == other.report_focus
            && self.disable_bracketed_paste_mode == other.disable_bracketed_paste_mode
            && self.mouse_mode == other.mouse_mode
            && self.keyboard_enhancements == other.keyboard_enhancements
            && self.progress_bar == other.progress_bar
    }
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
            progress_bar: None,
        }
    }
}

impl View {
    /// <upstream-comment>NewView is a helper function to create a new [View] with the given styled
    /// string. A styled string represents text with styles and hyperlinks encoded
    /// as ANSI escape codes.
    ///
    /// ```text
    /// v := tea.NewView("Hello, World!")
    /// ```</upstream-comment>
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

impl View {
    /// Returns whether two views are equivalent for rendering purposes,
    /// mirroring the upstream `viewEquals` check used to skip re-renders.
    pub fn equals(&self, o: &View) -> bool {
        self.content == o.content
            && self.alt_screen == o.alt_screen
            && self.report_focus == o.report_focus
            && self.disable_bracketed_paste_mode == o.disable_bracketed_paste_mode
            && self.window_title == o.window_title
            && self.mouse_mode == o.mouse_mode
            && self.background_color == o.background_color
            && self.foreground_color == o.foreground_color
            && self.keyboard_enhancements == o.keyboard_enhancements
            && self.cursor == o.cursor
            && self.progress_bar == o.progress_bar
    }
}
