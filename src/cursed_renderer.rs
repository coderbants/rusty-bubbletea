//! Cleanroom Rust port of upstream Go source file: `cursed_renderer.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Cursed Renderer
//!
//! High-performance standard terminal renderer for Bubble Tea v2.0.8, managing declarative `View` frames, ANSI diffing, cursor repositioning, and queued unmanaged messages.
//! </public-docs>

use crate::model::Cmd;
use crate::mouse::MouseMsg;
use crate::renderer::Renderer;
use crate::view::{MouseMode, View};
use crossterm::{
    cursor::{Hide, MoveToPreviousLine, Show},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};
use std::io::{stdout, Write};
use std::sync::Mutex;

/// CursedRenderer manages high-performance declarative View rendering in Bubble Tea v2.0.8.
pub struct CursedRenderer {
    mtx: Mutex<()>,
    last_view: Option<View>,
    queued_message_lines: Vec<String>,
    lines_rendered: usize,
    width: usize,
    height: usize,
    alt_screen_active: bool,
}

impl CursedRenderer {
    /// Creates a new CursedRenderer.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            mtx: Mutex::new(()),
            last_view: None,
            queued_message_lines: Vec::new(),
            lines_rendered: 0,
            width,
            height,
            alt_screen_active: false,
        }
    }
}

impl Renderer for CursedRenderer {
    fn start(&mut self) {}

    fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let _guard = self.mtx.lock().unwrap();
        if self.alt_screen_active {
            let _ = execute!(stdout(), LeaveAlternateScreen);
            self.alt_screen_active = false;
        }
        let _ = execute!(stdout(), Clear(ClearType::CurrentLine));
        print!("\r");
        let _ = stdout().flush();
        Ok(())
    }

    fn render(&mut self, view: View) {
        let _guard = self.mtx.lock().unwrap();

        // 1. Toggles Alternate Screen
        if view.alt_screen != self.alt_screen_active {
            self.alt_screen_active = view.alt_screen;
            if self.alt_screen_active {
                let _ = execute!(stdout(), EnterAlternateScreen);
            } else {
                let _ = execute!(stdout(), LeaveAlternateScreen);
            }
        }

        // 2. Window Title
        if !view.window_title.is_empty() {
            let _ = execute!(stdout(), SetTitle(&view.window_title));
        }

        // 3. Mouse Mode
        match view.mouse_mode {
            MouseMode::MouseModeNone => {
                let _ = execute!(stdout(), crossterm::event::DisableMouseCapture);
            }
            MouseMode::MouseModeCellMotion | MouseMode::MouseModeAllMotion => {
                let _ = execute!(stdout(), crossterm::event::EnableMouseCapture);
            }
        }

        // 4. Cursor Visibility
        if view.cursor.is_some() {
            let _ = execute!(stdout(), Show);
        } else {
            let _ = execute!(stdout(), Hide);
        }

        // 5. Queued Messages Flush above TUI
        if !self.queued_message_lines.is_empty() && !self.alt_screen_active {
            for line in self.queued_message_lines.drain(..) {
                println!("\r{}", line);
            }
        }

        // 6. Draw Content Frame
        if self.lines_rendered > 1 {
            let _ = execute!(stdout(), MoveToPreviousLine((self.lines_rendered - 1) as u16));
        } else if self.lines_rendered == 1 {
            print!("\r");
        }

        let _ = execute!(stdout(), Clear(ClearType::FromCursorDown));

        if !view.content.is_empty() {
            print!("{}", view.content);
            let _ = stdout().flush();
            // Count the number of lines by counting newline characters.
            // A trailing \n means the cursor is on the next (blank) line,
            // which is what the Go implementation tracks.
            self.lines_rendered = view.content.chars().filter(|&c| c == '\n').count() + 1;
        } else {
            self.lines_rendered = 0;
        }

        self.last_view = Some(view);
    }

    fn flush(&mut self, _closing: bool) -> Result<(), Box<dyn std::error::Error>> {
        let _ = stdout().flush();
        Ok(())
    }

    fn reset(&mut self) {
        let _guard = self.mtx.lock().unwrap();
        self.lines_rendered = 0;
        self.last_view = None;
    }

    fn insert_above(&mut self, s: String) -> Result<(), Box<dyn std::error::Error>> {
        let _guard = self.mtx.lock().unwrap();
        for line in s.split('\n') {
            self.queued_message_lines.push(line.to_string());
        }
        Ok(())
    }

    fn resize(&mut self, width: usize, height: usize) {
        let _guard = self.mtx.lock().unwrap();
        self.width = width;
        self.height = height;
    }

    fn clear_screen(&mut self) {
        let _guard = self.mtx.lock().unwrap();
        let _ = execute!(stdout(), Clear(ClearType::All));
        self.lines_rendered = 0;
    }

    fn write_string(&mut self, s: &str) -> Result<usize, Box<dyn std::error::Error>> {
        print!("{}", s);
        let _ = stdout().flush();
        Ok(s.len())
    }

    fn on_mouse(&mut self, msg: MouseMsg) -> Cmd {
        let _guard = self.mtx.lock().unwrap();
        if let Some(ref view) = self.last_view {
            if let Some(ref handler) = view.on_mouse {
                return handler(msg);
            }
        }
        None
    }
}
