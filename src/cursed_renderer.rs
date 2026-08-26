//! Cleanroom Rust port of upstream Go source file: `cursed_renderer.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! The high-performance standard terminal renderer for Bubble Tea v2.0.8.
//!
//! Wraps [rusty_ultraviolet::TerminalRenderer] and
//! [rusty_ultraviolet::ScreenBuffer] exactly like the upstream `cursedRenderer`
//! struct, so the emitted ANSI output is byte-identical to Go. Manages
//! declarative `View` frames, alt-screen mode, cursor visibility, terminal
//! modes (bracketed paste, focus, mouse, kitty keyboard), window title, and
//! queued unmanaged messages.
//! </public-docs>

use crate::color::Color;
use crate::cursor::CursorShape;
use crate::keyboard::KeyboardEnhancements;
use crate::model::Cmd;
use crate::mouse::MouseMsg;
use crate::renderer::Renderer;
use crate::view::{MouseMode, ProgressBar, ProgressBarState, View};
use rusty_ultraviolet::{Environ, ScreenBuffer, StyledString};
use rusty_x_ansi as ansi;
use rusty_x_ansi::method::WidthMethod;
use std::io::Write;

/// CursedRenderer manages high-performance declarative View rendering.
pub struct CursedRenderer {
    w: Box<dyn Write + Send + Sync>,
    /// Updates buffer to be flushed to [Self::w].
    buf: Vec<u8>,
    scr: rusty_ultraviolet::TerminalRenderer,
    cellbuf: ScreenBuffer,
    last_view: Option<View>,
    env: Vec<String>,
    // NOTE: the upstream stores `term` ($TERM); it is only used for the
    // renderer construction which reads it from the env directly.
    #[allow(dead_code)]
    term: String,
    width: usize,
    height: usize,
    // NOTE: upstream guards methods with a `sync.Mutex`; the ported methods
    // take `&mut self`, so exclusivity is enforced by the type system.
    profile: rusty_colorprofile::Profile,
    logger: Option<Box<dyn rusty_ultraviolet::Logger + Send + Sync>>,
    view: View,
    /// Whether to use hard tabs to optimize cursor movements.
    hard_tabs: bool,
    /// Whether to use backspace to optimize cursor movements.
    backspace: bool,
    map_nl: bool,
    /// Whether to use synchronized output mode for updates.
    syncd_updates: bool,
    /// Indicates whether the renderer is starting after being stopped.
    starting: bool,
}

/// NewCursedRenderer creates a new [CursedRenderer].
pub fn new_cursed_renderer(
    w: Box<dyn Write + Send + Sync>,
    env: &[String],
    width: usize,
    height: usize,
) -> CursedRenderer {
    let mut s = CursedRenderer {
        w,
        buf: Vec::new(),
        scr: rusty_ultraviolet::TerminalRenderer::new_without_writer(&Environ(env.to_vec())),
        cellbuf: rusty_ultraviolet::new_screen_buffer(width, height),
        last_view: None,
        env: env.to_vec(),
        term: Environ(env.to_vec()).getenv("TERM"),
        width,
        height,
        profile: rusty_colorprofile::Profile::NoTty,
        logger: None,
        view: View::default(),
        hard_tabs: false,
        backspace: false,
        map_nl: false,
        syncd_updates: false,
        starting: false,
    };
    reset(&mut s);
    s
}

impl CursedRenderer {
    /// SetLogger sets the logger for the renderer.
    pub fn set_logger(&mut self, logger: Option<Box<dyn rusty_ultraviolet::Logger + Send + Sync>>) {
        self.logger = logger;
    }

    /// SetOptimizations sets the cursor movement optimizations.
    pub fn set_optimizations(&mut self, hard_tabs: bool, backspace: bool, map_nl: bool) {
        self.hard_tabs = hard_tabs;
        self.backspace = backspace;
        self.map_nl = map_nl;
        if self.hard_tabs {
            self.scr.set_tab_stops_public(self.width as i32);
        } else {
            self.scr.set_tab_stops_public(-1);
        }
        self.scr.set_backspace_public(self.backspace);
        self.scr.set_map_newline_public(self.map_nl);
    }

    /// SetColorProfile sets the color profile of the renderer.
    pub fn set_color_profile(&mut self, p: rusty_colorprofile::Profile) {
        self.profile = p;
        self.scr.set_color_profile_public(p);
    }

    /// SetSyncdUpdates sets whether synchronized output mode is used.
    pub fn set_syncd_updates(&mut self, syncd: bool) {
        self.syncd_updates = syncd;
    }

    /// SetWidthMethod sets the width method of the renderer.
    pub fn set_width_method(&mut self, method: WidthMethod) {
        if method == WidthMethod::GraphemeWidth {
            // Turn on Unicode mode (2027) for accurate grapheme width
            // calculation.
            self.scr
                .write_string_public(ansi::mode::SET_MODE_UNICODE_CORE)
                .ok();
        } else if self.cellbuf.method == WidthMethod::GraphemeWidth {
            // Turn off Unicode mode if we're switching away from grapheme
            // width calculation.
            self.scr
                .write_string_public(ansi::mode::RESET_MODE_UNICODE_CORE)
                .ok();
        }
        self.cellbuf.method = method;
    }

    /// SetScrollOptim sets whether to use hard scroll optimizations.
    pub fn set_scroll_optim(&mut self, v: bool) {
        self.scr.set_scroll_optim_public(v);
    }
}

/// reset reinitializes the internal screen renderer.
fn reset(s: &mut CursedRenderer) {
    s.buf.clear();
    s.scr = rusty_ultraviolet::TerminalRenderer::new_without_writer(&Environ(s.env.clone()));
    s.scr.set_color_profile_public(s.profile);
    s.scr.set_relative_cursor_public(true); // Always start in inline mode
    s.scr.set_fullscreen_public(false); // Always start in inline mode
    if s.hard_tabs {
        s.scr.set_tab_stops_public(s.width as i32);
    } else {
        s.scr.set_tab_stops_public(-1);
    }
    s.scr.set_backspace_public(s.backspace);
    s.scr.set_map_newline_public(s.map_nl);
    s.scr.set_scroll_optim_public(true); // disable on Windows upstream
}

/// EnableAltScreen sets the alt screen mode. Writes to the buffer directly if
/// write is true.
fn enable_alt_screen(s: &mut CursedRenderer, enable: bool, write: bool) {
    if enable {
        enter_alt_screen(s, write);
    } else {
        exit_alt_screen(s, write);
    }
}

fn enter_alt_screen(s: &mut CursedRenderer, write: bool) {
    s.scr.save_cursor_public();
    if write {
        let _ = s
            .scr
            .write_string_public(ansi::mode::SET_MODE_ALT_SCREEN_SAVE_CURSOR);
    }
    s.scr.set_fullscreen_public(true);
    s.scr.set_relative_cursor_public(false);
    s.scr.erase_public();
}

fn exit_alt_screen(s: &mut CursedRenderer, write: bool) {
    s.scr.erase_public();
    s.scr.set_relative_cursor_public(true);
    s.scr.set_fullscreen_public(false);
    if write {
        let _ = s
            .scr
            .write_string_public(ansi::mode::RESET_MODE_ALT_SCREEN_SAVE_CURSOR);
    }
    s.scr.restore_cursor_public();
}

/// EnableTextCursor sets the text cursor mode.
fn enable_text_cursor(s: &mut CursedRenderer, enable: bool) {
    if enable {
        let _ = s
            .scr
            .write_string_public(ansi::mode::SET_MODE_TEXT_CURSOR_ENABLE);
    } else {
        let _ = s
            .scr
            .write_string_public(ansi::mode::RESET_MODE_TEXT_CURSOR_ENABLE);
    }
}

/// SetProgressBar writes the progress bar sequence for the given progress
/// bar.
fn set_progress_bar(s: &mut CursedRenderer, pb: Option<&ProgressBar>) {
    match pb {
        None => {
            let _ = s
                .scr
                .write_string_public(ansi::progress::RESET_PROGRESS_BAR);
        }
        Some(pb) => {
            let seq = match pb.state {
                ProgressBarState::ProgressBarNone => ansi::progress::RESET_PROGRESS_BAR.to_string(),
                ProgressBarState::ProgressBarDefault => {
                    ansi::progress::set_progress_bar(pb.value as i32)
                }
                ProgressBarState::ProgressBarError => {
                    ansi::progress::set_error_progress_bar(pb.value as i32)
                }
                ProgressBarState::ProgressBarIndeterminate => {
                    ansi::progress::SET_INDETERMINATE_PROGRESS_BAR.to_string()
                }
                ProgressBarState::ProgressBarWarning => {
                    ansi::progress::set_warning_progress_bar(pb.value as i32)
                }
            };
            if !seq.is_empty() {
                let _ = s.scr.write_string_public(&seq);
            }
        }
    }
}

/// ViewEquals returns whether the two views are equal.
pub(crate) fn view_equals(a: &View, b: &View) -> bool {
    if a.content != b.content
        || a.alt_screen != b.alt_screen
        || a.disable_bracketed_paste_mode != b.disable_bracketed_paste_mode
        || a.report_focus != b.report_focus
        || a.mouse_mode != b.mouse_mode
        || a.window_title != b.window_title
        || a.foreground_color != b.foreground_color
        || a.background_color != b.background_color
        || a.keyboard_enhancements != b.keyboard_enhancements
    {
        return false;
    }

    if (a.cursor.is_none()) != (b.cursor.is_none()) {
        return false;
    }
    if let (Some(ac), Some(bc)) = (&a.cursor, &b.cursor) {
        if ac.position.x != bc.position.x
            || ac.position.y != bc.position.y
            || ac.shape != bc.shape
            || ac.blink != bc.blink
            || ac.color != bc.color
        {
            return false;
        }
    }

    if (a.progress_bar.is_none()) != (b.progress_bar.is_none()) {
        return false;
    }
    if let (Some(ap), Some(bp)) = (&a.progress_bar, &b.progress_bar) {
        if ap.state != bp.state || ap.value != bp.value {
            return false;
        }
    }

    true
}

/// KeyboardEnhancementsFlags returns the kitty keyboard enhancement flags.
fn keyboard_enhancements_flags(ke: &KeyboardEnhancements) -> i32 {
    let mut flags = 1; // always enable basic key disambiguation
    if ke.report_event_types {
        flags |= ansi::kitty::KITTY_REPORT_EVENT_TYPES as i32;
    }
    if ke.report_alternate_keys {
        flags |= ansi::kitty::KITTY_REPORT_ALTERNATE_KEYS as i32;
    }
    if ke.report_all_keys_as_escape_codes {
        flags |= ansi::kitty::KITTY_REPORT_ALL_KEYS_AS_ESCAPE_CODES as i32;
    }
    if ke.report_associated_text {
        flags |= ansi::kitty::KITTY_REPORT_ASSOCIATED_KEYS as i32;
    }
    flags
}

/// EncodeCursorStyle encodes the cursor shape and blink into the ANSI
/// sequence value.
fn encode_cursor_style(style: CursorShape, blink: bool) -> i32 {
    // We're using the ANSI escape sequence values for cursor styles.
    let mut s = (style as i32) * 2 + 1;
    if !blink {
        s += 1;
    }
    s
}

/// A terminal color update: (new color, old color, reset sequence, setter).
type ColorUpdate = (
    Option<Color>,
    Option<Color>,
    &'static str,
    fn(&str) -> String,
);

impl Renderer for CursedRenderer {
    fn set_optimizations(&mut self, hard_tabs: bool, backspace: bool, map_nl: bool) {
        CursedRenderer::set_optimizations(self, hard_tabs, backspace, map_nl);
    }

    fn set_color_profile(&mut self, p: rusty_colorprofile::Profile) {
        CursedRenderer::set_color_profile(self, p);
    }

    fn start(&mut self) {
        // Mark that we're starting. This is used to restore some state when
        // starting the renderer again after it was stopped.
        self.starting = true;

        let Some(lv) = self.last_view.clone() else {
            return;
        };

        if lv.alt_screen {
            enable_alt_screen(self, true, true);
        }
        enable_text_cursor(self, lv.cursor.is_some());
        if let Some(cur) = &lv.cursor {
            if let Some(col) = cur.color {
                let col: Color = col;
                let _ = self
                    .scr
                    .write_string_public(&ansi::background::set_cursor_color(&col.hex()));
            }
            let cur_style = encode_cursor_style(cur.shape, cur.blink);
            if cur_style != 0 && cur_style != 1 {
                let _ = self
                    .scr
                    .write_string_public(&ansi::cursor::set_cursor_style(cur_style));
            }
        }
        if let Some(col) = lv.foreground_color {
            let _ = self
                .scr
                .write_string_public(&ansi::background::set_foreground_color(&col.hex()));
        }
        if let Some(col) = lv.background_color {
            let _ = self
                .scr
                .write_string_public(&ansi::background::set_background_color(&col.hex()));
        }
        if !lv.disable_bracketed_paste_mode {
            let _ = self
                .scr
                .write_string_public(ansi::mode::SET_MODE_BRACKETED_PASTE);
        }
        if lv.report_focus {
            let _ = self
                .scr
                .write_string_public(ansi::mode::SET_MODE_FOCUS_EVENT);
        }
        match lv.mouse_mode {
            MouseMode::MouseModeNone => {}
            MouseMode::MouseModeCellMotion => {
                let _ = self.scr.write_string_public(
                    ansi::mode::SET_MODE_MOUSE_BUTTON_EVENT.to_owned().as_str(),
                );
                let _ = self
                    .scr
                    .write_string_public(ansi::mode::SET_MODE_MOUSE_EXT_SGR);
            }
            MouseMode::MouseModeAllMotion => {
                let _ = self
                    .scr
                    .write_string_public(ansi::mode::SET_MODE_MOUSE_ANY_EVENT);
                let _ = self
                    .scr
                    .write_string_public(ansi::mode::SET_MODE_MOUSE_EXT_SGR);
            }
        }
        if !lv.window_title.is_empty() {
            let _ = self
                .scr
                .write_string_public(&ansi::screen::set_window_title(&lv.window_title));
        }
        if lv.progress_bar.is_some() {
            set_progress_bar(self, lv.progress_bar.as_ref());
        }
        // Enable modifyOtherKeys and Kitty keyboard protocol.
        let _ = self
            .scr
            .write_string_public(ansi::mode::ENABLE_MODIFY_OTHER_KEYS2);

        let kitty_flags = keyboard_enhancements_flags(&lv.keyboard_enhancements);
        let _ = self
            .scr
            .write_string_public(&ansi::kitty::kitty_keyboard(kitty_flags as u8, 1));
    }

    fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Exit the altScreen and show cursor before closing. It's important
        // that we don't change the altScreen and cursorHidden states so that
        // we can restore them when we start the renderer again.
        if let Some(lv) = &self.last_view {
            let lv = lv.clone();
            // NOTE: The Kitty keyboard specs specify that the terminal should
            // have two registries for the main and alt screens. Here, we
            // reset the keyboard protocol of the last screen used.
            self.scr
                .write_string_public(ansi::mode::RESET_MODIFY_OTHER_KEYS)?;
            self.scr
                .write_string_public(&ansi::kitty::kitty_keyboard(0, 1))?;

            // Go to the bottom of the screen.
            self.scr.move_to_public(0, self.cellbuf.height() as i64 - 1);
            let mut out = Vec::new();
            self.scr.flush_into(&mut out);
            self.buf.extend_from_slice(&out);
            if lv.alt_screen {
                enable_alt_screen(self, false, true);
            } else {
                let _ = self
                    .scr
                    .write_string_public(ansi::screen::ERASE_SCREEN_BELOW);
            }
            if lv.cursor.is_none() {
                enable_text_cursor(self, true);
            }
            if !lv.disable_bracketed_paste_mode {
                let _ = self
                    .scr
                    .write_string_public(ansi::mode::RESET_MODE_BRACKETED_PASTE);
            }
            if lv.report_focus {
                let _ = self
                    .scr
                    .write_string_public(ansi::mode::RESET_MODE_FOCUS_EVENT);
            }
            match lv.mouse_mode {
                MouseMode::MouseModeNone => {}
                MouseMode::MouseModeCellMotion | MouseMode::MouseModeAllMotion => {
                    let _ = self
                        .scr
                        .write_string_public(ansi::mode::RESET_MODE_MOUSE_BUTTON_EVENT);
                    let _ = self
                        .scr
                        .write_string_public(ansi::mode::RESET_MODE_MOUSE_ANY_EVENT);
                    let _ = self
                        .scr
                        .write_string_public(ansi::mode::RESET_MODE_MOUSE_EXT_SGR);
                }
            }

            if !lv.window_title.is_empty() {
                // Clear the window title if it was set.
                let _ = self
                    .scr
                    .write_string_public(&ansi::screen::set_window_title(""));
            }
            if let Some(lc) = &lv.cursor {
                let cur_shape = encode_cursor_style(lc.shape, lc.blink);
                if cur_shape != 0 && cur_shape != 1 {
                    // Reset the cursor style to default.
                    let _ = self
                        .scr
                        .write_string_public(&ansi::cursor::set_cursor_style(0));
                }

                if lc.color.is_some() {
                    let _ = self
                        .scr
                        .write_string_public(ansi::background::RESET_CURSOR_COLOR);
                }
            }

            if lv.background_color.is_some() {
                let _ = self
                    .scr
                    .write_string_public(ansi::background::RESET_BACKGROUND_COLOR);
            }
            if lv.foreground_color.is_some() {
                let _ = self
                    .scr
                    .write_string_public(ansi::background::RESET_FOREGROUND_COLOR);
            }
            if let Some(pb) = &lv.progress_bar {
                if pb.state != ProgressBarState::ProgressBarNone {
                    let _ = self
                        .scr
                        .write_string_public(ansi::progress::RESET_PROGRESS_BAR);
                }
            }
        }

        if self.cellbuf.method == WidthMethod::GraphemeWidth {
            // Make sure to turn off Unicode mode (2027).
            let _ = self
                .scr
                .write_string_public(ansi::mode::RESET_MODE_UNICODE_CORE);
        }

        let mut out = Vec::new();
        self.scr.flush_into(&mut out);
        self.buf.extend_from_slice(&out);

        if !self.buf.is_empty() {
            self.w.write_all(&self.buf)?;
            self.buf.clear();
        }

        let (x, y) = self.scr.position_public();

        // We want to clear the renderer state but not the cursor position.
        reset(self);
        self.scr.set_position_public(x, y);

        Ok(())
    }

    fn render(&mut self, view: View) {
        self.view = view;
    }

    fn flush(&mut self, closing: bool) -> Result<(), Box<dyn std::error::Error>> {
        let view = self.view.clone();
        let mut frame_area = rusty_ultraviolet::rect(0, 0, self.width, self.height);
        if view.content.is_empty() {
            // If the component is nil, we should clear the screen buffer.
            frame_area.max.1 = 0;
        }

        let content = StyledString {
            text: view.content.clone(),
            ..StyledString::default()
        };
        if !view.alt_screen {
            // We need to resize the screen based on the frame height and
            // terminal width.
            let frame_height = content.height();
            if frame_height != frame_area.dy() {
                frame_area.max.1 = frame_height;
            }
        }

        // Restore tab stops if we have tab optimizations enabled.
        if self.starting && self.hard_tabs {
            let _ = self
                .scr
                .write_string_public(ansi::screen::SET_TAB_EVERY_8_COLUMNS);
        }

        if !self.starting
            && !closing
            && self.last_view.is_some()
            && view_equals(self.last_view.as_ref().unwrap(), &view)
            && frame_area == self.cellbuf.bounds()
        {
            // No changes, nothing to do.
            return Ok(());
        }

        // We're no longer starting.
        self.starting = false;

        if frame_area != self.cellbuf.bounds() {
            self.scr.erase_public(); // Force a full redraw to avoid artifacts.

            // We need to reset the touched lines buffer to match the new
            // height.
            self.cellbuf.render_buffer.touched.clear();

            // Resize the screen buffer to match the frame area.
            self.cellbuf
                .render_buffer
                .buffer
                .resize(frame_area.dx(), frame_area.dy());
        }

        // Clear our screen buffer before copying the new frame into it to
        // ensure we erase any old content.
        self.cellbuf.render_buffer.clear();
        let bounds = self.cellbuf.bounds();
        content.draw(&mut self.cellbuf, bounds);

        // If the frame height is greater than the screen height, we drop the
        // lines from the top of the buffer.
        let frame_height = frame_area.dy();
        if frame_height > self.height {
            let drop = frame_height - self.height;
            self.cellbuf.render_buffer.buffer.lines.drain(..drop);
        }

        // Alt screen mode.
        let should_update_alt_screen = (self.last_view.is_none() && view.alt_screen)
            || (self.last_view.is_some()
                && self.last_view.as_ref().unwrap().alt_screen != view.alt_screen);
        if should_update_alt_screen {
            enable_alt_screen(self, view.alt_screen, false);
        }

        // bracketed paste mode.
        if self.last_view.is_none()
            || view.disable_bracketed_paste_mode
                != self
                    .last_view
                    .as_ref()
                    .unwrap()
                    .disable_bracketed_paste_mode
        {
            if !view.disable_bracketed_paste_mode {
                let _ = self
                    .scr
                    .write_string_public(ansi::mode::SET_MODE_BRACKETED_PASTE);
            } else if self.last_view.is_some() {
                let _ = self
                    .scr
                    .write_string_public(ansi::mode::RESET_MODE_BRACKETED_PASTE);
            }
        }

        // report focus events mode.
        if self.last_view.is_none()
            || self.last_view.as_ref().unwrap().report_focus != view.report_focus
        {
            if view.report_focus {
                let _ = self
                    .scr
                    .write_string_public(ansi::mode::SET_MODE_FOCUS_EVENT);
            } else if self.last_view.is_some() {
                let _ = self
                    .scr
                    .write_string_public(ansi::mode::RESET_MODE_FOCUS_EVENT);
            }
        }

        // mouse events mode.
        let last_mouse = self.last_view.as_ref().map(|v| v.mouse_mode);
        if self.last_view.is_none() || last_mouse != Some(view.mouse_mode) {
            match view.mouse_mode {
                MouseMode::MouseModeNone => {
                    if last_mouse.is_some() && last_mouse != Some(MouseMode::MouseModeNone) {
                        let _ = self
                            .scr
                            .write_string_public(ansi::mode::RESET_MODE_MOUSE_BUTTON_EVENT);
                        let _ = self
                            .scr
                            .write_string_public(ansi::mode::RESET_MODE_MOUSE_ANY_EVENT);
                        let _ = self
                            .scr
                            .write_string_public(ansi::mode::RESET_MODE_MOUSE_EXT_SGR);
                    }
                }
                MouseMode::MouseModeCellMotion => {
                    if last_mouse == Some(MouseMode::MouseModeAllMotion) {
                        let _ = self
                            .scr
                            .write_string_public(ansi::mode::RESET_MODE_MOUSE_ANY_EVENT);
                    }
                    let _ = self
                        .scr
                        .write_string_public(ansi::mode::SET_MODE_MOUSE_BUTTON_EVENT);
                    let _ = self
                        .scr
                        .write_string_public(ansi::mode::SET_MODE_MOUSE_EXT_SGR);
                }
                MouseMode::MouseModeAllMotion => {
                    if last_mouse == Some(MouseMode::MouseModeCellMotion) {
                        let _ = self
                            .scr
                            .write_string_public(ansi::mode::RESET_MODE_MOUSE_BUTTON_EVENT);
                    }
                    let _ = self
                        .scr
                        .write_string_public(ansi::mode::SET_MODE_MOUSE_ANY_EVENT);
                    let _ = self
                        .scr
                        .write_string_public(ansi::mode::SET_MODE_MOUSE_EXT_SGR);
                }
            }
        }

        // Set window title.
        let last_title = self.last_view.as_ref().map(|v| v.window_title.clone());
        if (self.last_view.is_none() || last_title.as_deref() != Some(view.window_title.as_str()))
            && (self.last_view.is_some() || !view.window_title.is_empty())
        {
            let _ = self
                .scr
                .write_string_public(&ansi::screen::set_window_title(&view.window_title));
        }

        // kitty keyboard protocol
        let last_ke = self.last_view.as_ref().map(|v| v.keyboard_enhancements);
        let last_alt = self.last_view.as_ref().map(|v| v.alt_screen);
        if self.last_view.is_none()
            || last_ke.as_ref() != Some(&view.keyboard_enhancements)
            || last_alt != Some(view.alt_screen)
        {
            // Enable modifyOtherKeys and Kitty keyboard protocol.
            let _ = self
                .scr
                .write_string_public(ansi::mode::ENABLE_MODIFY_OTHER_KEYS2);

            let kitty_flags = keyboard_enhancements_flags(&view.keyboard_enhancements);
            let _ = self
                .scr
                .write_string_public(&ansi::kitty::kitty_keyboard(kitty_flags as u8, 1));
            if !closing {
                // Request keyboard enhancements when they change.
                let _ = self
                    .scr
                    .write_string_public(ansi::kitty::REQUEST_KITTY_KEYBOARD);
            }
        }

        // Set terminal colors.
        let cc = view.cursor.as_ref().and_then(|c| c.color);
        let lcc = self
            .last_view
            .as_ref()
            .and_then(|v| v.cursor.as_ref())
            .and_then(|c| c.color);
        let lfg = self.last_view.as_ref().and_then(|v| v.foreground_color);
        let lbg = self.last_view.as_ref().and_then(|v| v.background_color);
        let colors: [ColorUpdate; 3] = [
            (
                cc,
                lcc,
                ansi::background::RESET_CURSOR_COLOR,
                ansi::background::set_cursor_color,
            ),
            (
                view.foreground_color,
                lfg,
                ansi::background::RESET_FOREGROUND_COLOR,
                ansi::background::set_foreground_color,
            ),
            (
                view.background_color,
                lbg,
                ansi::background::RESET_BACKGROUND_COLOR,
                ansi::background::set_background_color,
            ),
        ];
        for (new_color, old_color, reset, setter) in colors {
            if new_color != old_color {
                match new_color {
                    None => {
                        // Reset the color if it was set to nil.
                        let _ = self.scr.write_string_public(reset);
                    }
                    Some(col) => {
                        // Set the color.
                        let _ = self.scr.write_string_public(&setter(&col.hex()));
                    }
                }
            }
        }

        // Set cursor shape and blink if set.
        let cc_style = view
            .cursor
            .as_ref()
            .map(|c| encode_cursor_style(c.shape, c.blink));
        let lc_style = self
            .last_view
            .as_ref()
            .and_then(|v| v.cursor.as_ref())
            .map(|c| encode_cursor_style(c.shape, c.blink));
        if cc_style != lc_style {
            let _ = self
                .scr
                .write_string_public(&ansi::cursor::set_cursor_style(cc_style.unwrap_or(0)));
        }

        // Render progress bar if it's changed.
        let last_pb = self.last_view.as_ref().and_then(|v| v.progress_bar);
        let view_pb = view.progress_bar;
        let pb_changed = (self.last_view.is_none()
            && view_pb.is_some()
            && view_pb.map(|p| p.state) != Some(ProgressBarState::ProgressBarNone))
            || (self.last_view.is_some() && (last_pb.is_none()) != (view_pb.is_none()))
            || (last_pb.is_some() && view_pb.is_some() && last_pb != view_pb);
        if pb_changed {
            set_progress_bar(self, view_pb.as_ref());
        }

        // Render and queue changes to the screen buffer.
        self.scr.render_public(&mut self.cellbuf.render_buffer);

        if let Some(cur) = &view.cursor {
            // MoveTo must come after Render because the cursor position might
            // get updated during rendering.
            self.scr
                .move_to_public(cur.position.x as i64, cur.position.y as i64);
        } else if !view.alt_screen {
            // We don't want the cursor to be dangling at the end of the line
            // in inline mode.
            let (x, y) = self.scr.position_public();
            if x >= self.width.saturating_sub(1) {
                self.scr.move_to_public(0, y as i64);
            }
        }

        let mut out = Vec::new();
        self.scr.flush_into(&mut out);
        self.buf.extend_from_slice(&out);

        // Check if we have any render updates to flush.
        let has_updates = !self.buf.is_empty();

        // Cursor visibility.
        let did_show_cursor = self
            .last_view
            .as_ref()
            .map(|v| v.cursor.is_some())
            .unwrap_or(false);
        let show_cursor = view.cursor.is_some();
        let hide_cursor = !show_cursor;
        let should_update_cursor_vis = (self.last_view.is_none() || did_show_cursor != show_cursor)
            || should_update_alt_screen;

        // Build final output buffer with synchronized output or hide/show
        // cursor updates. But first, enter/exit alt screen mode if needed.
        let mut buf: Vec<u8> = Vec::new();
        if should_update_alt_screen {
            // We always disable keyboard enhancements when switching screens.
            let _ = ansi::mode::RESET_MODIFY_OTHER_KEYS;
            buf.extend_from_slice(ansi::mode::RESET_MODIFY_OTHER_KEYS.as_bytes());
            buf.extend_from_slice(ansi::kitty::kitty_keyboard(0, 1).as_bytes());
            if view.alt_screen {
                buf.extend_from_slice(ansi::mode::SET_MODE_ALT_SCREEN_SAVE_CURSOR.as_bytes());
            } else {
                buf.extend_from_slice(ansi::mode::RESET_MODE_ALT_SCREEN_SAVE_CURSOR.as_bytes());
            }
        }

        if self.syncd_updates {
            if has_updates {
                buf.extend_from_slice(ansi::mode::SET_MODE_SYNCHRONIZED_OUTPUT.as_bytes());
            }
            if should_update_cursor_vis && hide_cursor {
                buf.extend_from_slice(ansi::mode::RESET_MODE_TEXT_CURSOR_ENABLE.as_bytes());
            }
        } else if (should_update_cursor_vis && hide_cursor)
            || (has_updates && show_cursor && did_show_cursor)
        {
            buf.extend_from_slice(ansi::mode::RESET_MODE_TEXT_CURSOR_ENABLE.as_bytes());
        }

        if has_updates {
            buf.extend_from_slice(&self.buf);
        }

        if self.syncd_updates {
            if should_update_cursor_vis && show_cursor {
                buf.extend_from_slice(ansi::mode::SET_MODE_TEXT_CURSOR_ENABLE.as_bytes());
            }
            if has_updates {
                buf.extend_from_slice(ansi::mode::RESET_MODE_SYNCHRONIZED_OUTPUT.as_bytes());
            }
        } else if (should_update_cursor_vis && show_cursor)
            || (has_updates && show_cursor && did_show_cursor)
        {
            buf.extend_from_slice(ansi::mode::SET_MODE_TEXT_CURSOR_ENABLE.as_bytes());
        }

        // Reset internal screen renderer buffer.
        self.buf.clear();

        // If our updates flush buffer has content, write it to the output
        // writer.
        if std::env::var("UV_RENDER_DEBUG").is_ok() && !buf.is_empty() {
            use std::io::Write as _;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/flush.log")
                .unwrap();
            let _ = writeln!(
                f,
                "FLUSH ({}) {:?}",
                buf.len(),
                String::from_utf8_lossy(&buf)
            );
        }
        if !buf.is_empty() {
            self.w.write_all(&buf)?;
            // Rust's stdout is line-buffered: without an explicit flush the
            // tail of a frame after a newline stays buffered and never
            // reaches the terminal until the next newline-terminated write
            // (the upstream os.Stdout is unbuffered). Flush every frame.
            self.w.flush()?;
        }

        self.last_view = Some(view);

        Ok(())
    }

    fn reset(&mut self) {
        reset(self);
    }

    fn insert_above(&mut self, str_: String) -> Result<(), Box<dyn std::error::Error>> {
        if str_.is_empty() {
            return Ok(());
        }

        let mut sb = String::new();
        let (w, h) = (self.cellbuf.width(), self.cellbuf.height());
        let (_, y) = self.scr.position_public();

        // We need to scroll the screen up by the number of lines in the
        // queue.
        sb.push('\r');
        let down = h as i64 - y as i64 - 1;
        if down > 0 {
            sb.push_str(&ansi::cursor::cursor_down(down as i32));
        }

        let lines: Vec<&str> = str_.split('\n').collect();
        let mut offset = lines.len();
        for line in &lines {
            let line_width = ansi::width::string_width(line);
            if w > 0 && line_width > w {
                offset += line_width / w;
            }
        }

        // Scroll the screen up by the offset to make room for the new lines.
        sb.push_str(&"\n".repeat(offset));

        // XXX: Now go to the top of the screen, insert new lines, and write
        // the queued strings.
        let up = offset + h - 1;
        sb.push_str(&ansi::cursor::cursor_up(up as i32));
        sb.push_str(&ansi::screen::insert_line(offset as i32));
        for line in &lines {
            sb.push_str(line);
            sb.push_str(ansi::screen::ERASE_LINE_RIGHT);
            sb.push_str("\r\n");
        }

        self.scr.set_position_public(0, 0);

        self.w.write_all(sb.as_bytes())?;

        Ok(())
    }

    fn resize(&mut self, width: usize, height: usize) {
        // We need to mark the screen for clear to force a redraw.
        self.scr.erase_public();
        self.width = width;
        self.height = height;
        self.scr.resize_public(width, height);
    }

    fn clear_screen(&mut self) {
        // Move the cursor to the top left corner of the screen and trigger a
        // full screen redraw.
        self.scr.move_to_public(0, 0);
        self.scr.erase_public();
    }

    fn write_string(&mut self, s: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let n = s.len();
        self.scr.write_string_public(s)?;
        Ok(n)
    }

    fn write_direct(&mut self, s: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let bytes = s.as_bytes();
        self.w.write_all(bytes)?;
        self.w.flush()?;
        Ok(bytes.len())
    }

    fn on_mouse(&mut self, m: MouseMsg) -> Cmd {
        if let Some(lv) = &self.last_view {
            if let Some(on_mouse) = &lv.on_mouse {
                return on_mouse(m);
            }
        }
        None
    }
}

#[allow(dead_code)]
fn _assert_send<T: Send>(_: &T) {}
