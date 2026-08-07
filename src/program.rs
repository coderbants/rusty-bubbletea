//! Cleanroom Rust port of upstream Go source file: `tea.go` (Program runner)
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

use crate::commands::{BatchMsg, QuitMsg, RequestWindowSizeMsg, SequenceMsg};
use crate::cursed_renderer::CursedRenderer;
use crate::exec::ExecMsg;
use crate::key::{Key, KeyMod, KeyPressMsg, KEY_BACKSPACE, KEY_DOWN, KEY_END, KEY_ENTER, KEY_ESCAPE, KEY_HOME, KEY_LEFT, KEY_PG_DOWN, KEY_PG_UP, KEY_RIGHT, KEY_TAB, KEY_UP};
use crate::model::{Model, Msg};
use crate::mouse::{Mouse, MouseButton, MouseClickMsg, MouseReleaseMsg, MouseWheelMsg};
use crate::options::ProgramOptions;
use crate::renderer::Renderer;
use crate::screen::{ClearScreenMsg, WindowSizeMsg};
use crossterm::{
    event::{self, Event as CrossEvent, KeyCode, KeyModifiers, MouseEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, size as term_size},
};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Program is the runner for a Bubble Tea v2.0.8 application.
pub struct Program<M: Model> {
    model: M,
    options: ProgramOptions<M>,
    renderer: Box<dyn Renderer>,
}

impl<M: Model> Program<M> {
    /// Creates a new Program for the given model with default options.
    pub fn new(model: M) -> Self {
        let (w, h) = term_size().unwrap_or((80, 24));
        Self {
            model,
            options: ProgramOptions::default(),
            renderer: Box::new(CursedRenderer::new(w as usize, h as usize)),
        }
    }

    /// Sets custom program options.
    pub fn with_options(mut self, options: ProgramOptions<M>) -> Self {
        self.options = options;
        self
    }

    /// Helper to process a message, execute terminal commands, and dispatch generated commands.
    fn handle_msg(&mut self, msg: Box<dyn Msg>, tx: &Sender<Box<dyn Msg>>) -> bool {
        let processed_msg = if let Some(ref filter) = self.options.filter {
            match filter(&self.model, msg) {
                Some(m) => m,
                None => return false,
            }
        } else {
            msg
        };

        if processed_msg.as_ref().as_any().is::<QuitMsg>() {
            return true;
        }

        if processed_msg.as_ref().as_any().is::<ClearScreenMsg>() {
            self.renderer.clear_screen();
        } else if processed_msg.as_ref().as_any().is::<RequestWindowSizeMsg>() {
            if let Ok((w, h)) = term_size() {
                let _ = tx.send(Box::new(WindowSizeMsg {
                    width: w as usize,
                    height: h as usize,
                }));
            }
        } else if let Some(ws) = processed_msg.as_ref().as_any().downcast_ref::<WindowSizeMsg>() {
            self.renderer.resize(ws.width, ws.height);
        } else if let Some(exec_msg) = processed_msg.as_ref().as_any().downcast_ref::<ExecMsg>() {
            let _ = disable_raw_mode();
            let mut cmd = std::process::Command::new(&exec_msg.cmd);
            cmd.args(&exec_msg.args);
            let _ = cmd.status();
            let _ = enable_raw_mode();
        }

        if let Some(_batch) = processed_msg.as_ref().as_any().downcast_ref::<BatchMsg>() {
            return false;
        }

        if let Some(_seq) = processed_msg.as_ref().as_any().downcast_ref::<SequenceMsg>() {
            return false;
        }

        let cmd = self.model.update(processed_msg);
        let view = self.model.view();
        self.renderer.render(view);

        if let Some(c) = cmd {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                if let Some(new_msg) = c() {
                    let _ = tx_clone.send(new_msg);
                }
            });
        }
        false
    }

    /// Runs the Bubble Tea v2.0.8 event loop until quit.
    pub fn run(mut self) -> Result<M, Box<dyn std::error::Error>> {
        let _ = enable_raw_mode();
        self.renderer.start();

        let (tx, rx): (Sender<Box<dyn Msg>>, Receiver<Box<dyn Msg>>) = channel();

        // Spawn Crossterm input listener thread
        let event_tx = tx.clone();
        thread::spawn(move || loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(evt) = event::read() {
                    match evt {
                        CrossEvent::Key(k) => {
                            let mut mod_keys = KeyMod::default();
                            if k.modifiers.contains(KeyModifiers::CONTROL) {
                                mod_keys.0 |= KeyMod::CTRL.0;
                            }
                            if k.modifiers.contains(KeyModifiers::ALT) {
                                mod_keys.0 |= KeyMod::ALT.0;
                            }
                            if k.modifiers.contains(KeyModifiers::SHIFT) {
                                mod_keys.0 |= KeyMod::SHIFT.0;
                            }

                            let (code, text) = match k.code {
                                KeyCode::Char(c) => (c, c.to_string()),
                                KeyCode::Enter => (KEY_ENTER, String::new()),
                                KeyCode::Backspace => (KEY_BACKSPACE, String::new()),
                                KeyCode::Tab => (KEY_TAB, String::new()),
                                KeyCode::Esc => (KEY_ESCAPE, String::new()),
                                KeyCode::Up => (KEY_UP, String::new()),
                                KeyCode::Down => (KEY_DOWN, String::new()),
                                KeyCode::Right => (KEY_RIGHT, String::new()),
                                KeyCode::Left => (KEY_LEFT, String::new()),
                                KeyCode::Home => (KEY_HOME, String::new()),
                                KeyCode::End => (KEY_END, String::new()),
                                KeyCode::PageUp => (KEY_PG_UP, String::new()),
                                KeyCode::PageDown => (KEY_PG_DOWN, String::new()),
                                _ => ('\0', String::new()),
                            };

                            let key = Key::new(code, &text, mod_keys);
                            let msg: Box<dyn Msg> = Box::new(KeyPressMsg(key));
                            let _ = event_tx.send(msg);
                        }
                        CrossEvent::Mouse(m) => {
                            let (button, is_release, is_wheel) = match m.kind {
                                MouseEventKind::Down(b) => (
                                    match b {
                                        crossterm::event::MouseButton::Left => MouseButton::MouseLeft,
                                        crossterm::event::MouseButton::Right => MouseButton::MouseRight,
                                        crossterm::event::MouseButton::Middle => MouseButton::MouseMiddle,
                                    },
                                    false,
                                    false,
                                ),
                                MouseEventKind::Up(b) => (
                                    match b {
                                        crossterm::event::MouseButton::Left => MouseButton::MouseLeft,
                                        crossterm::event::MouseButton::Right => MouseButton::MouseRight,
                                        crossterm::event::MouseButton::Middle => MouseButton::MouseMiddle,
                                    },
                                    true,
                                    false,
                                ),
                                MouseEventKind::ScrollUp => (MouseButton::MouseWheelUp, false, true),
                                MouseEventKind::ScrollDown => (MouseButton::MouseWheelDown, false, true),
                                _ => (MouseButton::MouseNone, false, false),
                            };

                            let mouse_data = Mouse {
                                x: m.column as usize,
                                y: m.row as usize,
                                button,
                                mod_keys: KeyMod::default(),
                            };

                            let msg: Box<dyn Msg> = if is_wheel {
                                Box::new(MouseWheelMsg(mouse_data))
                            } else if is_release {
                                Box::new(MouseReleaseMsg(mouse_data))
                            } else {
                                Box::new(MouseClickMsg(mouse_data))
                            };
                            let _ = event_tx.send(msg);
                        }
                        CrossEvent::Resize(w, h) => {
                            let msg: Box<dyn Msg> = Box::new(WindowSizeMsg {
                                width: w as usize,
                                height: h as usize,
                            });
                            let _ = event_tx.send(msg);
                        }
                        _ => {}
                    }
                }
            }
        });

        // Run initial command
        if let Some(cmd) = self.model.init() {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                if let Some(msg) = cmd() {
                    let _ = tx_clone.send(msg);
                }
            });
        }

        // Send initial window size query
        if let Ok((w, h)) = term_size() {
            let _ = tx.send(Box::new(WindowSizeMsg {
                width: w as usize,
                height: h as usize,
            }));
        }

        // Render initial view frame
        let initial_view = self.model.view();
        self.renderer.render(initial_view);

        // Main event processing loop
        while let Ok(msg) = rx.recv() {
            if self.handle_msg(msg, &tx) {
                break;
            }
        }

        let _ = self.renderer.close();
        let _ = disable_raw_mode();
        Ok(self.model)
    }
}
