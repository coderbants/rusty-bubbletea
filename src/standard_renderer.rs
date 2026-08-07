//! Cleanroom Rust port of upstream Go source file: `standard_renderer.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Standard Renderer
//!
//! StandardRenderer is a 1:1 port of upstream standard_renderer.go, managing framerate buffering,
//! diff-based line repainting, line clearing, and cursor positioning across render cycles.
//! </public-docs>

use crate::commands::WindowSizeMsg;
use crate::model::Msg;
use crate::renderer::Renderer;
use crate::screen::PrintlnMsg;
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
};
use std::io::{stdout, Write};
use std::sync::Mutex;
use std::time::Duration;

const DEFAULT_FPS: u32 = 60;
const MAX_FPS: u32 = 120;

/// <upstream-comment>
/// StandardRenderer is a framerate-based terminal renderer matching standard_renderer.go.
/// </upstream-comment>
pub struct StandardRenderer {
    mtx: Mutex<()>,
    buf: String,
    queued_message_lines: Vec<String>,
    /// Target framerate duration.
    pub framerate: Duration,
    last_render: String,
    last_rendered_lines: Vec<String>,
    lines_rendered: usize,
    alt_lines_rendered: usize,
    cursor_hidden: bool,
    alt_screen_active: bool,
    width: u16,
    height: u16,
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
            mtx: Mutex::new(()),
            buf: String::new(),
            queued_message_lines: Vec::new(),
            framerate: Duration::from_millis(1000 / effective_fps as u64),
            last_render: String::new(),
            last_rendered_lines: Vec::new(),
            lines_rendered: 0,
            alt_lines_rendered: 0,
            cursor_hidden: false,
            alt_screen_active: false,
            width: 0,
            height: 0,
        }
    }

    fn last_lines_rendered(&self) -> usize {
        if self.alt_screen_active {
            self.alt_lines_rendered
        } else {
            self.lines_rendered
        }
    }

    /// Cleanroom 1:1 port of standard_renderer.go `flush()` method.
    fn flush(&mut self) {
        let _guard = self.mtx.lock().unwrap();

        if self.buf.is_empty() || self.buf == self.last_render {
            return;
        }

        let mut output = String::new();

        // 1. Position cursor back to top of previously rendered section.
        if self.alt_screen_active {
            output.push_str("\x1b[1;1H");
        } else if self.lines_rendered > 1 {
            output.push_str(&format!("\x1b[{}A", self.lines_rendered - 1));
        }

        let mut new_lines: Vec<String> = self.buf.split('\n').map(|s| s.to_string()).collect();

        if self.height > 0 && new_lines.len() > self.height as usize {
            let start = new_lines.len() - self.height as usize;
            new_lines = new_lines[start..].to_vec();
        }

        let flush_queued = !self.queued_message_lines.is_empty() && !self.alt_screen_active;

        // 2. Output queued message lines (tea.Printf / tea.Println).
        if flush_queued {
            for line in self.queued_message_lines.drain(..) {
                output.push_str(&line);
                output.push_str("\x1b[K\r\n");
            }
        }

        // 3. Paint lines.
        for i in 0..new_lines.len() {
            let can_skip = !flush_queued
                && self.last_rendered_lines.len() > i
                && self.last_rendered_lines[i] == new_lines[i];

            if can_skip {
                if i < new_lines.len() - 1 {
                    output.push('\n');
                }
                continue;
            }

            if i == 0 && self.last_render.is_empty() {
                output.push('\r');
            }

            let mut line = new_lines[i].clone();

            if self.width > 0 && line.len() > self.width as usize {
                line.truncate(self.width as usize);
            }

            output.push_str(&line);
            output.push_str("\x1b[K");

            if i < new_lines.len() - 1 {
                output.push_str("\r\n");
            }
        }

        // 4. Clear leftover lines below.
        if self.last_lines_rendered() > new_lines.len() {
            output.push_str("\x1b[J");
        }

        if self.alt_screen_active {
            self.alt_lines_rendered = new_lines.len();
            output.push_str(&format!("\x1b[{};1H", new_lines.len()));
        } else {
            self.lines_rendered = new_lines.len();
            output.push_str("\r");
        }

        print!("{}", output);
        let _ = stdout().flush();

        self.last_rendered_lines = new_lines;
        self.last_render = self.buf.clone();
    }
}

impl Renderer for StandardRenderer {
    fn start(&mut self) {}

    fn stop(&mut self) {
        self.flush();
        let _guard = self.mtx.lock().unwrap();
        print!("\x1b[2K\r");
        let _ = stdout().flush();
    }

    fn kill(&mut self) {
        let _guard = self.mtx.lock().unwrap();
        print!("\x1b[2K\r");
        let _ = stdout().flush();
    }

    fn write(&mut self, s: String) {
        {
            let _guard = self.mtx.lock().unwrap();
            let content = if s.is_empty() { " ".to_string() } else { s };
            self.buf = content;
        }
        self.flush();
    }

    fn handle_message(&mut self, msg: &dyn Msg) {
        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            let _guard = self.mtx.lock().unwrap();
            self.width = ws.width;
            self.height = ws.height;
            self.last_render.clear();
            self.last_rendered_lines.clear();
        } else if let Some(println_msg) = msg.as_any().downcast_ref::<PrintlnMsg>() {
            if !self.alt_screen_active {
                let _guard = self.mtx.lock().unwrap();
                for line in println_msg.0.split('\n') {
                    self.queued_message_lines.push(line.to_string());
                }
                self.last_render.clear();
                self.last_rendered_lines.clear();
            }
        }
    }

    fn repaint(&mut self) {
        let _guard = self.mtx.lock().unwrap();
        self.last_render.clear();
        self.last_rendered_lines.clear();
    }

    fn clear_screen(&mut self) {
        let _guard = self.mtx.lock().unwrap();
        let _ = execute!(stdout(), Clear(ClearType::All));
        self.last_render.clear();
        self.last_rendered_lines.clear();
    }

    fn alt_screen(&self) -> bool {
        self.alt_screen_active
    }

    fn enter_alt_screen(&mut self) {
        let _guard = self.mtx.lock().unwrap();
        if self.alt_screen_active {
            return;
        }
        self.alt_screen_active = true;
        let _ = execute!(stdout(), EnterAlternateScreen);
        self.alt_lines_rendered = 0;
        self.last_render.clear();
        self.last_rendered_lines.clear();
    }

    fn exit_alt_screen(&mut self) {
        let _guard = self.mtx.lock().unwrap();
        if !self.alt_screen_active {
            return;
        }
        self.alt_screen_active = false;
        let _ = execute!(stdout(), LeaveAlternateScreen);
        self.last_render.clear();
        self.last_rendered_lines.clear();
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
