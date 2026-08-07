//! Cleanroom Rust port of upstream Go source file: `standard_renderer.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Standard Renderer
//!
//! StandardRenderer is a framerate-based terminal renderer updating the view to avoid overloading terminal emulators.
//! </public-docs>

use crate::commands::WindowSizeMsg;
use crate::model::Msg;
use crate::renderer::Renderer;
use crate::screen::PrintlnMsg;
use crossterm::{
    cursor::{Hide, MoveToPreviousLine, Show},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
};
use std::io::{stdout, Write};
use std::time::Duration;

const DEFAULT_FPS: u32 = 60;
const MAX_FPS: u32 = 120;

/// <upstream-comment>
/// StandardRenderer is a framerate-based terminal renderer.
/// </upstream-comment>
pub struct StandardRenderer {
    lines_rendered: usize,
    alt_screen_active: bool,
    cursor_hidden: bool,
    queued_message_lines: Vec<String>,
    width: u16,
    height: u16,
    /// Framerate interval for flushing output.
    pub framerate: Duration,
}

impl StandardRenderer {
    /// Creates a new StandardRenderer instance with given target fps.
    pub fn new(fps: u32) -> Self {
        let effective_fps = if fps == 0 {
            DEFAULT_FPS
        } else if fps > MAX_FPS {
            MAX_FPS
        } else {
            fps
        };

        Self {
            lines_rendered: 0,
            alt_screen_active: false,
            cursor_hidden: false,
            queued_message_lines: Vec::new(),
            width: 0,
            height: 0,
            framerate: Duration::from_millis(1000 / effective_fps as u64),
        }
    }
}

impl Renderer for StandardRenderer {
    fn start(&mut self) {}

    fn stop(&mut self) {
        let _ = execute!(stdout(), Clear(ClearType::CurrentLine));
        print!("\r");
        let _ = stdout().flush();
    }

    fn kill(&mut self) {
        self.stop();
    }

    fn write(&mut self, s: String) {
        let flush_queued = !self.queued_message_lines.is_empty() && !self.alt_screen_active;

        if flush_queued {
            for line in self.queued_message_lines.drain(..) {
                println!("\r{}", line);
            }
        }

        if self.lines_rendered > 1 {
            let _ = execute!(stdout(), MoveToPreviousLine((self.lines_rendered - 1) as u16));
        } else if self.lines_rendered == 1 {
            print!("\r");
        }

        let _ = execute!(stdout(), Clear(ClearType::FromCursorDown));

        if !s.is_empty() {
            print!("{}", s);
            let _ = stdout().flush();
            self.lines_rendered = s.lines().count().max(1);
        } else {
            self.lines_rendered = 0;
        }
    }

    fn handle_message(&mut self, msg: &dyn Msg) {
        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            self.height = ws.height;
            self.repaint();
        } else if let Some(println_msg) = msg.as_any().downcast_ref::<PrintlnMsg>() {
            if !self.alt_screen_active {
                for line in println_msg.0.split('\n') {
                    let s: String = line.to_string();
                    self.queued_message_lines.push(s);
                }
                self.repaint();
            }
        }
    }

    fn repaint(&mut self) {
        self.lines_rendered = 0;
    }

    fn clear_screen(&mut self) {
        let _ = execute!(stdout(), Clear(ClearType::All));
        self.repaint();
    }

    fn alt_screen(&self) -> bool {
        self.alt_screen_active
    }

    fn enter_alt_screen(&mut self) {
        if self.alt_screen_active {
            return;
        }
        self.alt_screen_active = true;
        let _ = execute!(stdout(), EnterAlternateScreen);
        self.repaint();
    }

    fn exit_alt_screen(&mut self) {
        if !self.alt_screen_active {
            return;
        }
        self.alt_screen_active = false;
        let _ = execute!(stdout(), LeaveAlternateScreen);
        self.repaint();
    }

    fn show_cursor(&mut self) {
        self.cursor_hidden = false;
        let _ = execute!(stdout(), Show);
    }

    fn hide_cursor(&mut self) {
        self.cursor_hidden = true;
        let _ = execute!(stdout(), Hide);
    }

    fn enable_mouse_cell_motion(&mut self) {
        let _ = execute!(stdout(), crossterm::event::EnableMouseCapture);
    }

    fn disable_mouse_cell_motion(&mut self) {
        let _ = execute!(stdout(), crossterm::event::DisableMouseCapture);
    }

    fn enable_mouse_all_motion(&mut self) {
        let _ = execute!(stdout(), crossterm::event::EnableMouseCapture);
    }

    fn disable_mouse_all_motion(&mut self) {
        let _ = execute!(stdout(), crossterm::event::DisableMouseCapture);
    }

    fn enable_mouse_sgr_mode(&mut self) {}

    fn disable_mouse_sgr_mode(&mut self) {}

    fn enable_bracketed_paste(&mut self) {}

    fn disable_bracketed_paste(&mut self) {}

    fn bracketed_paste_active(&self) -> bool {
        false
    }

    fn set_window_title(&mut self, title: &str) {
        let _ = execute!(stdout(), SetTitle(title));
    }

    fn report_focus(&self) -> bool {
        false
    }

    fn enable_report_focus(&mut self) {}

    fn disable_report_focus(&mut self) {}
}
