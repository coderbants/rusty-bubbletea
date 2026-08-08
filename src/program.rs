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

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::commands::{BatchMsg, InterruptMsg, QuitMsg, RequestWindowSizeMsg, ResumeMsg, SequenceMsg, SuspendMsg};
use crate::cursed_renderer::CursedRenderer;
use crate::environ::EnvMsg;
use crate::exec::ExecMsg;
use crate::key::{Key, KeyMod, KeyPressMsg, KEY_BACKSPACE, KEY_DOWN, KEY_END, KEY_ENTER, KEY_ESCAPE, KEY_HOME, KEY_LEFT, KEY_PG_DOWN, KEY_PG_UP, KEY_RIGHT, KEY_TAB, KEY_UP};
use crate::model::{Model, Msg};
use crate::mouse::{Mouse, MouseButton, MouseClickMsg, MouseReleaseMsg, MouseWheelMsg};
use crate::options::ProgramOptions;
use crate::profile::ColorProfileMsg;
use crate::renderer::{PrintLineMsg, Renderer};
use crate::screen::{ClearScreenMsg, WindowSizeMsg};
use crossterm::{
    event::{self, Event as CrossEvent, KeyCode, KeyModifiers, MouseEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, size as term_size},
};

/// <upstream-comment>ErrProgramPanic is returned by [Program.Run] when the program recovers from a panic.</upstream-comment>
pub const ERR_PROGRAM_PANIC: &str = "program experienced a panic";

/// <upstream-comment>ErrProgramKilled is returned by [Program.Run] when the program gets killed.</upstream-comment>
pub const ERR_PROGRAM_KILLED: &str = "program was killed";

/// <upstream-comment>ErrInterrupted is returned by [Program.Run] when the program get a SIGINT
/// signal, or when it receives a [InterruptMsg].</upstream-comment>
pub const ERR_INTERRUPTED: &str = "program was interrupted";

/// ProgramError is the error type returned by [Program::run].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramError {
    /// The program was killed (context cancellation or external kill).
    Killed,
    /// The program was interrupted (SIGINT or [InterruptMsg]).
    Interrupted,
    /// The program recovered from a panic.
    Panic,
}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramError::Killed => write!(f, "{}", ERR_PROGRAM_KILLED),
            ProgramError::Interrupted => write!(f, "{}", ERR_INTERRUPTED),
            ProgramError::Panic => write!(f, "{}", ERR_PROGRAM_PANIC),
        }
    }
}

impl std::error::Error for ProgramError {}

/// Program is the runner for a Bubble Tea v2.0.8 application.
pub struct Program<M: Model> {
    model: M,
    options: ProgramOptions<M>,
    renderer: Box<dyn Renderer>,
    msg_tx: Option<Sender<Box<dyn Msg>>>,
    finished: Arc<AtomicBool>,
}

impl<M: Model> Program<M> {
    /// Creates a new Program for the given model with default options.
    pub fn new(model: M) -> Self {
        let (w, h) = term_size().unwrap_or((80, 24));
        Self {
            model,
            options: ProgramOptions::default(),
            renderer: Box::new(CursedRenderer::new(w as usize, h as usize)),
            msg_tx: None,
            finished: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Sets custom program options.
    pub fn with_options(mut self, options: ProgramOptions<M>) -> Self {
        self.options = options;
        self
    }

    /// <upstream-comment>Send sends a message to the main update function, effectively allowing
    /// messages to be injected from outside the program for interoperability
    /// purposes.</upstream-comment>
    pub fn send(&self, msg: Box<dyn Msg>) {
        if let Some(tx) = &self.msg_tx {
            let _ = tx.send(msg);
        }
    }

    /// <upstream-comment>Quit is a convenience function for quitting Bubble Tea programs. Use it
    /// when you need to shut down a Bubble Tea program from the outside.</upstream-comment>
    pub fn quit(&self) {
        self.send(Box::new(QuitMsg));
    }

    /// <upstream-comment>Kill stops the program immediately and restores the former terminal state.
    /// The final render that you would normally see when quitting will be skipped.
    /// [Program.Run] returns a [ErrProgramKilled] error.</upstream-comment>
    pub fn kill(&mut self) {
        // Disable raw mode and mark the program finished; the run loop observes
        // the finished flag and exits.
        let _ = disable_raw_mode();
        self.finished.store(true, Ordering::SeqCst);
        if let Some(tx) = &self.msg_tx {
            let _ = tx.send(Box::new(QuitMsg));
        }
    }

    /// <upstream-comment>Wait waits/blocks until the underlying Program finished shutting down.</upstream-comment>
    pub fn wait(&self) {
        while !self.finished.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// <upstream-comment>Println prints above the Program. This output is unmanaged by the program
    /// and will persist across renders by the Program.</upstream-comment>
    pub fn println(&self, args: &str) {
        self.send(Box::new(PrintLineMsg {
            message_body: args.to_string(),
        }));
    }

    /// <upstream-comment>Printf prints above the Program. It takes a format template followed by
    /// values similar to fmt.Printf.</upstream-comment>
    pub fn printf(&self, body: &str) {
        self.send(Box::new(PrintLineMsg {
            message_body: body.to_string(),
        }));
    }

    /// Helper to process a message, execute terminal commands, and dispatch generated commands.
    fn handle_msg(&mut self, msg: Box<dyn Msg>, tx: &Sender<Box<dyn Msg>>) -> Result<bool, ProgramError> {
        let processed_msg = if let Some(ref filter) = self.options.filter {
            match filter(&self.model, msg) {
                Some(m) => m,
                None => return Ok(false),
            }
        } else {
            msg
        };

        if processed_msg.as_ref().as_any().is::<QuitMsg>() {
            return Ok(true);
        }

        if processed_msg.as_ref().as_any().is::<InterruptMsg>() {
            return Err(ProgramError::Interrupted);
        }

        if processed_msg.as_ref().as_any().is::<SuspendMsg>() {
            // Best-effort suspension: restore the terminal until a resume
            // message arrives; the program continues afterwards.
            let _ = disable_raw_mode();
            self.send_resume_later(tx);
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
            // RequestWindowSizeMsg itself is internal — don't pass to model.update
            return Ok(false);
        } else if let Some(ws) = processed_msg.as_ref().as_any().downcast_ref::<WindowSizeMsg>() {
            // Resize the renderer first, then fall through to model.update below
            self.renderer.resize(ws.width, ws.height);
        } else if let Some(exec_msg) = processed_msg.as_ref().as_any().downcast_ref::<ExecMsg>() {
            let _ = disable_raw_mode();
            let mut cmd = std::process::Command::new(&exec_msg.cmd);
            cmd.args(&exec_msg.args);
            let _ = cmd.status();
            let _ = enable_raw_mode();
        } else if let Some(print_msg) = processed_msg.as_ref().as_any().downcast_ref::<PrintLineMsg>() {
            // Insert the line above the TUI without routing through model.update
            let _ = self.renderer.insert_above(print_msg.message_body.clone());
            // Re-render to flush queued lines
            let view = self.model.view();
            self.renderer.render(view);
            return Ok(false);
        } else if let Some(env) = processed_msg.as_ref().as_any().downcast_ref::<EnvMsg>() {
            let _ = env;
        } else if let Some(profile) = processed_msg.as_ref().as_any().downcast_ref::<ColorProfileMsg>() {
            let _ = profile;
        } else if let Some(_resume) = processed_msg.as_ref().as_any().downcast_ref::<ResumeMsg>() {
            let _ = enable_raw_mode();
        }

        if let Some(_batch) = processed_msg.as_ref().as_any().downcast_ref::<BatchMsg>() {
            return Ok(false);
        }

        if let Some(_seq) = processed_msg.as_ref().as_any().downcast_ref::<SequenceMsg>() {
            return Ok(false);
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
        Ok(false)
    }

    fn send_resume_later(&self, tx: &Sender<Box<dyn Msg>>) {
        let tx = tx.clone();
        thread::spawn(move || {
            let _ = tx.send(Box::new(ResumeMsg));
        });
    }

    /// Runs the Bubble Tea v2.0.8 event loop until quit.
    pub fn run(mut self) -> Result<M, Box<dyn std::error::Error>> {
        let _ = enable_raw_mode();
        self.renderer.start();

        let (tx, rx): (Sender<Box<dyn Msg>>, Receiver<Box<dyn Msg>>) = channel();
        self.msg_tx = Some(tx.clone());

        let external_ctx = self.options.context.clone();

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

        // Send the environment variables used by the program.
        let _ = tx.send(Box::new(EnvMsg::from_std()));

        // Render initial view frame
        let initial_view = self.model.view();
        self.renderer.render(initial_view);

        // Main event processing loop
        let result = loop {
            // Check for external context cancellation.
            if let Some(ctx) = &external_ctx {
                if ctx.done() {
                    break Err(ProgramError::Killed);
                }
            }
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(msg) => match self.handle_msg(msg, &tx) {
                    Ok(true) => break Ok(()),
                    Ok(false) => continue,
                    Err(e) => break Err(e),
                },
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break Ok(()),
            }
        };

        self.finished.store(true, Ordering::SeqCst);
        let _ = self.renderer.close();
        let _ = disable_raw_mode();

        match result {
            Ok(()) => Ok(self.model),
            Err(e) => Err(Box::new(e)),
        }
    }
}
