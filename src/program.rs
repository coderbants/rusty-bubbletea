//! Cleanroom Rust port of upstream Go source file: `program.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Program
//!
//! Program runner managing the Elm architecture event loop, terminal raw mode, Crossterm event polling, rendering, and async command dispatches.
//! </public-docs>

use crate::commands::{
    BatchMsg, EnterAltScreenMsg, ExitAltScreenMsg, QuitMsg, RequestWindowSizeMsg, SequenceMsg,
    SetWindowTitleMsg, WindowSizeMsg,
};
use crate::key::{KeyMsg, KeyType};
use crate::model::{Model, Msg};
use crate::mouse::{MouseAction, MouseButton, MouseMsg};
use crate::renderer::Renderer;
use crate::standard_renderer::StandardRenderer;

use crossterm::{
    event::{self, Event as CrossEvent, KeyCode, KeyModifiers, MouseEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, size as term_size},
};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Event filter function closure type.
pub type FilterFn<M> = Box<dyn Fn(&M, Box<dyn Msg>) -> Option<Box<dyn Msg>> + Send + Sync>;

/// <upstream-comment>
/// Program is the runner for a Bubble Tea application.
/// </upstream-comment>
pub struct Program<M: Model> {
    model: M,
    /// Renderer instance.
    pub renderer: Box<dyn Renderer>,
    /// Optional event filter.
    pub filter: Option<FilterFn<M>>,
    enable_mouse: bool,
}

impl<M: Model> Program<M> {
    /// <upstream-comment>
    /// NewProgram creates a new Program instance for the given Model.
    /// </upstream-comment>
    pub fn new(model: M) -> Self {
        Self {
            model,
            renderer: Box::new(StandardRenderer::new(60)),
            filter: None,
            enable_mouse: false,
        }
    }

    /// Custom constructor accepting a custom renderer implementation.
    pub fn with_renderer(model: M, renderer: Box<dyn Renderer>) -> Self {
        Self {
            model,
            renderer,
            filter: None,
            enable_mouse: false,
        }
    }

    /// Enable mouse tracking for mouse-aware applications.
    pub fn with_mouse(mut self) -> Self {
        self.enable_mouse = true;
        self
    }

    /// Helper to process a message, execute terminal commands, and dispatch generated commands.
    fn handle_msg(&mut self, msg: Box<dyn Msg>, tx: &Sender<Box<dyn Msg>>) -> bool {
        let processed_msg = if let Some(ref filter) = self.filter {
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

        self.renderer.handle_message(processed_msg.as_ref());

        if processed_msg.as_ref().as_any().is::<EnterAltScreenMsg>() {
            self.renderer.enter_alt_screen();
        } else if processed_msg.as_ref().as_any().is::<ExitAltScreenMsg>() {
            self.renderer.exit_alt_screen();
        } else if let Some(title_msg) = processed_msg.as_ref().as_any().downcast_ref::<SetWindowTitleMsg>() {
            self.renderer.set_window_title(&title_msg.0);
        } else if processed_msg.as_ref().as_any().is::<RequestWindowSizeMsg>() {
            if let Ok((w, h)) = term_size() {
                let _ = tx.send(Box::new(WindowSizeMsg::new(w, h)));
            }
        }

        if let Some(_batch_msg) = processed_msg.as_ref().as_any().downcast_ref::<BatchMsg>() {
            return false;
        }

        if let Some(_seq_msg) = processed_msg.as_ref().as_any().downcast_ref::<SequenceMsg>() {
            return false;
        }

        let cmd = self.model.update(processed_msg);
        self.renderer.write(self.model.view());

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

    /// <upstream-comment>
    /// Run initializes raw mode, starts terminal event polling, executes commands, and runs the event loop until quit.
    /// </upstream-comment>
    pub fn run(mut self) -> Result<M, Box<dyn std::error::Error>> {
        let _ = enable_raw_mode();
        self.renderer.start();
        if self.enable_mouse {
            self.renderer.enable_mouse_all_motion();
        }

        let (tx, rx): (Sender<Box<dyn Msg>>, Receiver<Box<dyn Msg>>) = channel();

        // Spawn terminal input reader thread
        let event_tx = tx.clone();
        thread::spawn(move || loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(evt) = event::read() {
                    match evt {
                        CrossEvent::Key(k) => {
                            let (key_type, runes) = match k.code {
                                KeyCode::Char(c) => (KeyType::KeyRunes, vec![c]),
                                KeyCode::Enter => (KeyType::KeyEnter, vec![]),
                                KeyCode::Backspace => (KeyType::KeyBackspace, vec![]),
                                KeyCode::Tab => (KeyType::KeyTab, vec![]),
                                KeyCode::Esc => (KeyType::KeyEsc, vec![]),
                                KeyCode::Up => (KeyType::KeyUp, vec![]),
                                KeyCode::Down => (KeyType::KeyDown, vec![]),
                                KeyCode::Right => (KeyType::KeyRight, vec![]),
                                KeyCode::Left => (KeyType::KeyLeft, vec![]),
                                KeyCode::Home => (KeyType::KeyHome, vec![]),
                                KeyCode::End => (KeyType::KeyEnd, vec![]),
                                KeyCode::PageUp => (KeyType::KeyPgUp, vec![]),
                                KeyCode::PageDown => (KeyType::KeyPgDown, vec![]),
                                KeyCode::Delete => (KeyType::KeyDelete, vec![]),
                                KeyCode::BackTab => (KeyType::KeyShiftTab, vec![]),
                                _ => (KeyType::KeyUnknown, vec![]),
                            };

                            let is_alt = k.modifiers.contains(KeyModifiers::ALT);
                            let is_ctrl = k.modifiers.contains(KeyModifiers::CONTROL);

                            let msg: Box<dyn Msg> = if is_ctrl && runes == vec!['c'] {
                                Box::new(KeyMsg::new(KeyType::KeyCtrlC))
                            } else if key_type == KeyType::KeyRunes {
                                Box::new(KeyMsg::from_runes(&runes, is_alt))
                            } else {
                                Box::new(KeyMsg::new(key_type))
                            };

                            let _ = event_tx.send(msg);
                        }
                        CrossEvent::Mouse(m) => {
                            let (button, action) = match m.kind {
                                MouseEventKind::Down(b) => (
                                    match b {
                                        crossterm::event::MouseButton::Left => MouseButton::MouseLeft,
                                        crossterm::event::MouseButton::Right => MouseButton::MouseRight,
                                        crossterm::event::MouseButton::Middle => MouseButton::MouseMiddle,
                                    },
                                    MouseAction::MouseActionPress,
                                ),
                                MouseEventKind::Up(b) => (
                                    match b {
                                        crossterm::event::MouseButton::Left => MouseButton::MouseLeft,
                                        crossterm::event::MouseButton::Right => MouseButton::MouseRight,
                                        crossterm::event::MouseButton::Middle => MouseButton::MouseMiddle,
                                    },
                                    MouseAction::MouseActionRelease,
                                ),
                                MouseEventKind::Drag(b) => (
                                    match b {
                                        crossterm::event::MouseButton::Left => MouseButton::MouseLeft,
                                        crossterm::event::MouseButton::Right => MouseButton::MouseRight,
                                        crossterm::event::MouseButton::Middle => MouseButton::MouseMiddle,
                                    },
                                    MouseAction::MouseActionMotion,
                                ),
                                MouseEventKind::Moved => (
                                    MouseButton::MouseUnknown,
                                    MouseAction::MouseActionMotion,
                                ),
                                MouseEventKind::ScrollUp => (
                                    MouseButton::MouseWheelUp,
                                    MouseAction::MouseActionPress,
                                ),
                                MouseEventKind::ScrollDown => (
                                    MouseButton::MouseWheelDown,
                                    MouseAction::MouseActionPress,
                                ),
                                _ => (MouseButton::MouseUnknown, MouseAction::MouseActionMotion),
                            };

                            let mouse_msg = MouseMsg::new(m.column, m.row, button, action);
                            let _ = event_tx.send(Box::new(mouse_msg));
                        }
                        CrossEvent::Resize(w, h) => {
                            let _ = event_tx.send(Box::new(WindowSizeMsg::new(w, h)));
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
            let _ = tx.send(Box::new(WindowSizeMsg::new(w, h)));
        }

        // Initial view render
        self.renderer.write(self.model.view());

        // Event loop processing
        while let Ok(msg) = rx.recv() {
            if self.handle_msg(msg, &tx) {
                break;
            }
        }

        if self.enable_mouse {
            self.renderer.disable_mouse_all_motion();
        }
        self.renderer.stop();
        let _ = disable_raw_mode();
        Ok(self.model)
    }
}
