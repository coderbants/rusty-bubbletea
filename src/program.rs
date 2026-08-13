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

/// Message channel used to feed the program's event loop.
type MsgChannel = Sender<Box<dyn Msg>>;
/// Message channel used to receive messages into the program's event loop.
type MsgReceiver = Receiver<Box<dyn Msg>>;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::commands::{
    BatchMsg, InterruptMsg, QuitMsg, RequestWindowSizeMsg, ResumeMsg, SequenceMsg, SuspendMsg,
};
use crate::cursor::CursorPositionMsg;
use crate::environ::EnvMsg;
use crate::exec::ExecMsg;
use crate::focus::{BlurMsg, FocusMsg};
use crate::key::{KeyMod, KeyPressMsg, KeyReleaseMsg};
use crate::keyboard::KeyboardEnhancementsMsg;
use crate::model::{Model, Msg};
use crate::mouse::{MouseClickMsg, MouseMotionMsg, MouseReleaseMsg, MouseWheelMsg};
use crate::options::ProgramOptions;
use crate::paste::PasteMsg;
use crate::profile::ColorProfileMsg;
use crate::renderer::{PrintLineMsg, Renderer};
use crate::screen::{ClearScreenMsg, WindowSizeMsg};
use crate::tty::{disable_raw_mode, enable_raw_mode};
use crossterm::terminal::size as term_size;

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
    /// Buffered startup query sequences, flushed with the first render
    /// (mirrors upstream `p.outputBuf` + `p.execute`).
    startup_buf: Arc<Mutex<Option<Vec<u8>>>>,
    model: M,
    options: ProgramOptions<M>,
    renderer: Arc<Mutex<Box<dyn Renderer>>>,
    msg_tx: Option<MsgChannel>,
    finished: Arc<AtomicBool>,
}

impl<M: Model> Program<M> {
    /// Creates a new Program for the given model with default options.
    pub fn new(model: M) -> Self {
        let (w, h) = term_size().unwrap_or((80, 24));
        let env: Vec<String> = std::env::vars().map(|(k, v)| format!("{k}={v}")).collect();
        Self {
            startup_buf: Arc::new(Mutex::new(None)),
            model,
            options: ProgramOptions::default(),
            renderer: Arc::new(Mutex::new(Box::new(
                crate::cursed_renderer::new_cursed_renderer(
                    Box::new(std::io::stdout()),
                    &env,
                    w as usize,
                    h as usize,
                ),
            ))),
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
    fn handle_msg(&mut self, msg: Box<dyn Msg>, tx: &MsgChannel) -> Result<bool, ProgramError> {
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
            self.renderer.lock().unwrap().clear_screen();
        } else if processed_msg
            .as_ref()
            .as_any()
            .is::<crate::color::RequestBackgroundColorMsg>()
        {
            // Mirror upstream `p.execute(ansi.RequestBackgroundColor)`: the
            // query is buffered and flushed with the first render.
            if let Ok(mut buf) = self.startup_buf.lock() {
                if let Some(b) = buf.as_mut() {
                    b.extend_from_slice(
                        rusty_x_ansi::background::REQUEST_BACKGROUND_COLOR.as_bytes(),
                    );
                }
            }
        } else if processed_msg
            .as_ref()
            .as_any()
            .is::<crate::color::RequestForegroundColorMsg>()
        {
            if let Ok(mut buf) = self.startup_buf.lock() {
                if let Some(b) = buf.as_mut() {
                    b.extend_from_slice(
                        rusty_x_ansi::background::REQUEST_FOREGROUND_COLOR.as_bytes(),
                    );
                }
            }
        } else if processed_msg
            .as_ref()
            .as_any()
            .is::<crate::color::RequestCursorColorMsg>()
        {
            if let Ok(mut buf) = self.startup_buf.lock() {
                if let Some(b) = buf.as_mut() {
                    b.extend_from_slice(
                        rusty_x_ansi::background::REQUEST_CURSOR_COLOR.as_bytes(),
                    );
                }
            }
        } else if let Some(cap) = processed_msg
            .as_ref()
            .as_any()
            .downcast_ref::<crate::termcap::RequestCapabilityMsg>()
        {
            // Mirror upstream `p.execute(ansi.RequestTermcap(cap))`: write the
            // XTGETTCAP query (DCS + q <Pt> ST) to the terminal so the terminal
            // responds with a CapabilityMsg.
            use std::io::Write as _;
            let mut seq = String::from("\x1bP+q");
            for b in cap.0.as_bytes() {
                seq.push_str(&format!("{:02X}", b));
            }
            seq.push_str("\x1b\\");
            let _ = std::io::stdout().write_all(seq.as_bytes());
            return Ok(false);
        } else if processed_msg
            .as_ref()
            .as_any()
            .is::<crate::xterm::RequestTerminalVersionMsg>()
        {
            // Mirror upstream `p.execute(ansi.RequestNameVersion)`: query the
            // terminal name and version (XTVERSION) so the terminal responds
            // with a TerminalVersionMsg.
            use std::io::Write as _;
            let _ = std::io::stdout().write_all(b"\x1b[>q");
            return Ok(false);
        } else if processed_msg.as_ref().as_any().is::<RequestWindowSizeMsg>() {
            if let Ok((w, h)) = term_size() {
                let _ = tx.send(Box::new(WindowSizeMsg {
                    width: w as usize,
                    height: h as usize,
                }));
            }
            // RequestWindowSizeMsg itself is internal — don't pass to model.update
            return Ok(false);
        } else if let Some(ws) = processed_msg
            .as_ref()
            .as_any()
            .downcast_ref::<WindowSizeMsg>()
        {
            // Resize the renderer first, then fall through to model.update below
            self.renderer.lock().unwrap().resize(ws.width, ws.height);
        } else if let Some(exec_msg) = processed_msg.as_ref().as_any().downcast_ref::<ExecMsg>() {
            let _ = disable_raw_mode();
            let mut cmd = std::process::Command::new(&exec_msg.cmd);
            cmd.args(&exec_msg.args);
            let _ = cmd.status();
            let _ = enable_raw_mode();
        } else if let Some(print_msg) = processed_msg
            .as_ref()
            .as_any()
            .downcast_ref::<PrintLineMsg>()
        {
            // Insert the line above the TUI without routing through model.update
            let _ = self
                .renderer
                .lock()
                .unwrap()
                .insert_above(print_msg.message_body.clone());
            // Re-render to flush queued lines
            let view = self.model.view();
            self.renderer.lock().unwrap().render(view);
            return Ok(false);
        } else if let Some(env) = processed_msg.as_ref().as_any().downcast_ref::<EnvMsg>() {
            let _ = env;
        } else if let Some(profile) = processed_msg
            .as_ref()
            .as_any()
            .downcast_ref::<ColorProfileMsg>()
        {
            let p = match profile.profile {
                crate::profile::ColorProfile::TrueColor => {
                    rusty_colorprofile::Profile::TrueColor
                }
                crate::profile::ColorProfile::ANSI256 => rusty_colorprofile::Profile::Ansi256,
                crate::profile::ColorProfile::ANSI => rusty_colorprofile::Profile::Ansi,
                crate::profile::ColorProfile::Ascii => rusty_colorprofile::Profile::Ascii,
            };
            self.renderer.lock().unwrap().set_color_profile(p);
        } else if let Some(_resume) = processed_msg.as_ref().as_any().downcast_ref::<ResumeMsg>() {
            let _ = enable_raw_mode();
        }

        // Dispatch the commands carried by BatchMsg and SequenceMsg,
        // mirroring the upstream handling of `tea.Batch` and `tea.Sequence`
        // messages (`case BatchMsg: go p.execBatchMsg(msg); continue` and
        // `case sequenceMsg: go p.execSequenceMsg(msg); continue`): the
        // command trees are expanded on their own thread, recursively, so a
        // QuitMsg produced by a sequence is only delivered after every
        // preceding command (including nested batches and sequences) has
        // completed.
        if processed_msg.as_ref().as_any().is::<BatchMsg>() {
            let any = processed_msg.into_any();
            let batch = *any.downcast::<BatchMsg>().unwrap();
            let tx_clone = tx.clone();
            thread::spawn(move || exec_batch_msg(batch, &tx_clone));
            return Ok(false);
        }

        // Mirror upstream `case MouseMsg:` in the event loop: route mouse
        // messages to the renderer's on_mouse hook (used by composable view
        // layers) and send any produced message back through the program.
        // The message still falls through to the model's update below.
        let mouse_msg = {
            let any = processed_msg.as_ref().as_any();
            if let Some(m) = any.downcast_ref::<crate::mouse::MouseClickMsg>() {
                Some(crate::mouse::MouseMsg::Click(m.clone()))
            } else if let Some(m) = any.downcast_ref::<crate::mouse::MouseMotionMsg>() {
                Some(crate::mouse::MouseMsg::Motion(m.clone()))
            } else if let Some(m) = any.downcast_ref::<crate::mouse::MouseReleaseMsg>() {
                Some(crate::mouse::MouseMsg::Release(m.clone()))
            } else {
                any.downcast_ref::<crate::mouse::MouseWheelMsg>()
                    .map(|m| crate::mouse::MouseMsg::Wheel(m.clone()))
            }
        };
        if let Some(mouse_msg) = mouse_msg {
            let cmd = self.renderer.lock().unwrap().on_mouse(mouse_msg);
            if let Some(c) = cmd {
                let tx_clone = tx.clone();
                thread::spawn(move || {
                    if let Some(new_msg) = c() {
                        let _ = tx_clone.send(new_msg);
                    }
                });
            }
        }

        if processed_msg.as_ref().as_any().is::<SequenceMsg>() {
            let any = processed_msg.into_any();
            let seq = *any.downcast::<SequenceMsg>().unwrap();
            let tx_clone = tx.clone();
            thread::spawn(move || exec_sequence_msg(seq, &tx_clone));
            return Ok(false);
        }

        let cmd = self.model.update(&*processed_msg);
        let view = self.model.view();
        self.renderer.lock().unwrap().render(view);

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

    fn send_resume_later(&self, tx: &MsgChannel) {
        let tx = tx.clone();
        thread::spawn(move || {
            let _ = tx.send(Box::new(ResumeMsg));
        });
    }

    /// Runs the Bubble Tea v2.0.8 event loop until quit.
    pub fn run(mut self) -> Result<M, Box<dyn std::error::Error>> {
        let _ = enable_raw_mode();
        self.renderer.lock().unwrap().start();

        // Termios-based cursor movement optimizations, mirroring the
        // upstream `initInput` -> `checkOptimizedMovements` flow.
        // Detect the color profile from the environment and set it on the
        // renderer (upstream `colorprofile.Detect` at startup); the
        // ColorProfileMsg path may later upgrade it.
        {
            use std::os::fd::AsRawFd as _;
            let env = rusty_ultraviolet::Environ(
                std::env::vars().map(|(k, v)| format!("{k}={v}")).collect(),
            );
            let profile = rusty_ultraviolet::terminal_screen::detect_color_profile(
                Some(std::io::stdout().as_raw_fd()),
                &env,
            );
            self.renderer
                .lock()
                .unwrap()
                .set_color_profile(match profile {
                    rusty_ultraviolet::terminal_screen::ColorProfile::TrueColor => {
                        rusty_colorprofile::Profile::TrueColor
                    }
                    rusty_ultraviolet::terminal_screen::ColorProfile::Ansi256 => {
                        rusty_colorprofile::Profile::Ansi256
                    }
                    rusty_ultraviolet::terminal_screen::ColorProfile::Ansi => {
                        rusty_colorprofile::Profile::Ansi
                    }
                    _ => rusty_colorprofile::Profile::NoTty,
                });
        }

        let (hard_tabs, backspace) = check_optimized_movements();
        // mapNl is false when the input is a real TTY (upstream:
        // `runtime.GOOS != "windows" && p.ttyInput == nil`).
        let map_nl = false;
        self.renderer
            .lock()
            .unwrap()
            .set_optimizations(hard_tabs, backspace, map_nl);

        let (tx, rx): (MsgChannel, MsgReceiver) = channel();
        self.msg_tx = Some(tx.clone());

        let external_ctx = self.options.context.clone();

        // Input thread: reads raw bytes from stdin and decodes them through
        // the ultraviolet event decoder, mirroring the upstream
        // `uv.NewTerminalReader` input path.
        let input_tx = tx.clone();
        thread::spawn(move || {
            let reader: Box<dyn std::io::Read + Send> = Box::new(std::io::stdin());
            let mut tr = rusty_ultraviolet::terminal_reader::new_terminal_reader(
                reader,
                "xterm-256color",
            );
            tr.set_legacy(rusty_ultraviolet::LegacyKeyEncoding::default());
            let (dec_tx, dec_rx) = std::sync::mpsc::channel::<rusty_ultraviolet::DecodedEvent>();
            let streamer = std::thread::spawn(move || {
                let _ = tr.stream_events(&dec_tx);
            });
            for ev in dec_rx {
                if let Some(msg) = decoded_to_msg(ev) {
                    if input_tx.send(msg).is_err() {
                        break;
                    }
                }
            }
            let _ = streamer.join();
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

        // Send the detected color profile to the program, mirroring the
        // upstream `go p.Send(ColorProfileMsg{*p.profile})` at startup.
        {
            use std::os::fd::AsRawFd as _;
            let env = rusty_ultraviolet::Environ(
                std::env::vars().map(|(k, v)| format!("{k}={v}")).collect(),
            );
            let profile = rusty_ultraviolet::terminal_screen::detect_color_profile(
                Some(std::io::stdout().as_raw_fd()),
                &env,
            );
            let msg_profile = match profile {
                rusty_ultraviolet::terminal_screen::ColorProfile::TrueColor => {
                    crate::profile::ColorProfile::TrueColor
                }
                rusty_ultraviolet::terminal_screen::ColorProfile::Ansi256 => {
                    crate::profile::ColorProfile::ANSI256
                }
                rusty_ultraviolet::terminal_screen::ColorProfile::Ansi => {
                    crate::profile::ColorProfile::ANSI
                }
                _ => crate::profile::ColorProfile::Ascii,
            };
            let _ = tx.send(Box::new(crate::profile::ColorProfileMsg {
                profile: msg_profile,
            }));
        }

        // Query for synchronized updates support (mode 2026) and unicode core
        // (mode 2027), mirroring the upstream `p.execute(...)` at startup:
        // the queries are buffered and flushed together with the first
        // render (ticker flush or the quit path's flush(true)).
        let query_sync = should_query_synchronized_output();
        self.startup_buf = Arc::new(Mutex::new(if query_sync {
            Some(b"\x1b[?2026$p\x1b[?2027$p".to_vec())
        } else {
            None
        }));
        let startup_buf = self.startup_buf.clone();

        // Render initial view frame. The frame is flushed by the render
        // ticker (or the quit path's flush(true)), mirroring the upstream
        // ticker-driven render loop.
        let initial_view = self.model.view();
        self.renderer.lock().unwrap().render(initial_view);

        // Render ticker: flushes the pending view at the default framerate
        // (60fps), like the upstream `startRenderer` goroutine.
        let tick_renderer = self.renderer.clone();
        let done = self.finished.clone();
        let tick_buf = startup_buf.clone();
        thread::spawn(move || {
            let interval = Duration::from_millis(1000 / 60);
            while !done.load(Ordering::SeqCst) {
                thread::sleep(interval);
                if let Some(buf) = tick_buf.lock().unwrap().take() {
                    use std::io::Write as _;
                    let _ = std::io::stdout().write_all(&buf);
                }
                let _ = tick_renderer.lock().unwrap().flush(false);
            }
        });

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
        // Graceful shutdown: ensure we render the final state of the model
        // (upstream `p.render(model)` after the event loop).
        let final_view = self.model.view();
        self.renderer.lock().unwrap().render(final_view);
        // Flush the last frame with closing=true before closing, like the
        // upstream `stopRenderer` path. Note: any startup queries still
        // buffered are NOT written here — upstream flushes its output buffer
        // only from the render ticker goroutine, so queries buffered but
        // never flushed by the ticker are dropped (observed behavior).
        let _fr = self.renderer.lock().unwrap().flush(true);
        let _cr = self.renderer.lock().unwrap().close();
        let _ = disable_raw_mode();

        match result {
            Ok(()) => Ok(self.model),
            Err(e) => Err(Box::new(e)),
        }
    }
}

/// Execute the commands carried by a [BatchMsg], mirroring the upstream
/// `execBatchMsg` handling (`tea.go`): every command runs concurrently on its
/// own thread, and nested BatchMsg/SequenceMsg results are expanded inline
/// (recursively) instead of being routed back through the event loop.
/// Mirrors the upstream `wg.Wait()`: the batch is not complete until every
/// command has finished, so a sequence containing this batch blocks until
/// the whole tree completes.
fn exec_batch_msg(batch: BatchMsg, tx: &MsgChannel) {
    let handles: Vec<_> = batch
        .0
        .into_iter()
        .flatten()
        .map(|cmd| {
            let tx = tx.clone();
            thread::spawn(move || {
                if let Some(msg) = cmd() {
                    dispatch_msg(msg, &tx);
                }
            })
        })
        .collect();
    for handle in handles {
        let _ = handle.join();
    }
}

/// Execute the commands carried by a [SequenceMsg], mirroring the upstream
/// `execSequenceMsg` handling (`tea.go`): commands run one at a time in
/// order, on the calling thread, and nested BatchMsg/SequenceMsg results are
/// expanded inline (recursively) instead of being routed back through the
/// event loop.
fn exec_sequence_msg(seq: SequenceMsg, tx: &MsgChannel) {
    for cmd in seq.0.into_iter().flatten() {
        if let Some(msg) = cmd() {
            dispatch_msg(msg, tx);
        }
    }
}

/// Dispatch a command result message: nested [BatchMsg]/[SequenceMsg]
/// messages are expanded inline (recursively), while every other message is
/// sent back through the event loop channel — mirroring upstream
/// `execBatchMsg` / `execSequenceMsg` and `p.Send` for default messages.
fn dispatch_msg(msg: Box<dyn Msg>, tx: &MsgChannel) {
    if msg.as_ref().as_any().is::<BatchMsg>() {
        let any = msg.into_any();
        let batch = *any.downcast::<BatchMsg>().unwrap();
        exec_batch_msg(batch, tx);
    } else if msg.as_ref().as_any().is::<SequenceMsg>() {
        let any = msg.into_any();
        let seq = *any.downcast::<SequenceMsg>().unwrap();
        exec_sequence_msg(seq, tx);
    } else {
        let _ = tx.send(msg);
    }
}

/// ShouldQuerySynchronizedOutput returns whether the terminal is known to
/// support synchronized output (mode 2026), mirroring the upstream gate in
/// `tea.go`.
fn should_query_synchronized_output() -> bool {
    let term_type = std::env::var("TERM").unwrap_or_default();
    let term_prog = std::env::var("TERM_PROGRAM").ok();
    let ssh_tty = std::env::var("SSH_TTY").is_ok();
    let wt_session = std::env::var("WT_SESSION").is_ok();

    let ok_term_prog = term_prog.is_some();
    wt_session
        || term_type.contains("ghostty")
        || term_type.contains("wezterm")
        || (!ok_term_prog && !ssh_tty)
        || (!ssh_tty && !term_prog.as_deref().unwrap_or("").contains("Apple"))
        || term_type.contains("alacritty")
        || term_type.contains("kitty")
        || term_type.contains("rio")
}

/// Converts an ultraviolet [rusty_ultraviolet::DecodedEvent] into a
/// Bubble Tea message.
fn decoded_to_msg(ev: rusty_ultraviolet::DecodedEvent) -> Option<Box<dyn Msg>> {
    use rusty_ultraviolet::DecodedEvent as D;
    match ev {
        D::KeyPress(k) => Some(Box::new(KeyPressMsg(uv_key_to_key(k)))),
        D::KeyRelease(k) => Some(Box::new(KeyReleaseMsg(uv_key_to_key(k)))),
        D::MouseClick(m) => Some(Box::new(MouseClickMsg(uv_mouse_to_mouse(m)))),
        D::MouseRelease(m) => Some(Box::new(MouseReleaseMsg(uv_mouse_to_mouse(m)))),
        D::MouseWheel(m) => Some(Box::new(MouseWheelMsg(uv_mouse_to_mouse(m)))),
        D::MouseMotion(m) => Some(Box::new(MouseMotionMsg(uv_mouse_to_mouse(m)))),
        D::WindowSize(s) => Some(Box::new(WindowSizeMsg {
            width: s.width,
            height: s.height,
        })),
        D::Paste(s) => Some(Box::new(PasteMsg { content: s })),
        D::Focus => Some(Box::new(FocusMsg)),
        D::Blur => Some(Box::new(BlurMsg)),
        D::KeyboardEnhancements(flags) => Some(Box::new(KeyboardEnhancementsMsg { flags })),
        D::CursorPosition { x, y } => Some(Box::new(CursorPositionMsg {
            x: x.max(0) as usize,
            y: y.max(0) as usize,
        })),
        // Terminal query responses, mirroring the upstream
        // `uv.Event` -> `tea.Msg` translations (capability, name/version and
        // color responses).
        D::Capability(s) => Some(Box::new(crate::termcap::CapabilityMsg { content: s })),
        D::TerminalVersion(s) => Some(Box::new(crate::xterm::TerminalVersionMsg { name: s })),
        D::ForegroundColor(Some(c)) => Some(Box::new(crate::color::ForegroundColorMsg(c))),
        D::BackgroundColor(Some(c)) => Some(Box::new(crate::color::BackgroundColorMsg(c))),
        D::CursorColor(Some(c)) => Some(Box::new(crate::color::CursorColorMsg(c))),
        _ => None,
    }
}

/// Converts an ultraviolet key into the Bubble Tea key representation.
fn uv_key_to_key(k: rusty_ultraviolet::Key) -> crate::key::Key {
    use rusty_ultraviolet::key as uvk;
    let code = match k.code {
        uvk::KEY_UP => crate::key::KEY_UP,
        uvk::KEY_DOWN => crate::key::KEY_DOWN,
        uvk::KEY_LEFT => crate::key::KEY_LEFT,
        uvk::KEY_RIGHT => crate::key::KEY_RIGHT,
        uvk::KEY_PG_UP => crate::key::KEY_PG_UP,
        uvk::KEY_PG_DOWN => crate::key::KEY_PG_DOWN,
        uvk::KEY_HOME => crate::key::KEY_HOME,
        uvk::KEY_END => crate::key::KEY_END,
        uvk::KEY_ENTER => crate::key::KEY_ENTER,
        uvk::KEY_TAB => crate::key::KEY_TAB,
        uvk::KEY_BACKSPACE => crate::key::KEY_BACKSPACE,
        uvk::KEY_ESCAPE => crate::key::KEY_ESCAPE,
        uvk::KEY_SPACE => crate::key::KEY_SPACE,
        // Unmapped special keys fall back to the unicode replacement
        // character; the text field carries the printable representation.
        _ => char::from_u32(k.code).unwrap_or('\0'),
    };
    crate::key::Key {
        text: k.text.clone(),
        mod_keys: KeyMod(k.mod_.0 as u8),
        code,
        shifted_code: char::from_u32(k.shifted_code),
        base_code: char::from_u32(k.base_code),
        is_repeat: k.is_repeat,
    }
}

/// Converts an ultraviolet mouse event into the Bubble Tea representation.
fn uv_mouse_to_mouse(m: rusty_ultraviolet::Mouse) -> crate::mouse::Mouse {
    let button = match m.button.0 {
        0 => crate::mouse::MouseButton::MouseNone,
        1 => crate::mouse::MouseButton::MouseLeft,
        2 => crate::mouse::MouseButton::MouseMiddle,
        3 => crate::mouse::MouseButton::MouseRight,
        4 => crate::mouse::MouseButton::MouseWheelUp,
        5 => crate::mouse::MouseButton::MouseWheelDown,
        6 => crate::mouse::MouseButton::MouseWheelLeft,
        7 => crate::mouse::MouseButton::MouseWheelRight,
        8 => crate::mouse::MouseButton::MouseBackward,
        9 => crate::mouse::MouseButton::MouseForward,
        10 => crate::mouse::MouseButton::MouseButton10,
        _ => crate::mouse::MouseButton::MouseButton11,
    };
    crate::mouse::Mouse {
        x: m.x.max(0) as usize,
        y: m.y.max(0) as usize,
        button,
        mod_keys: KeyMod(m.mod_.0 as u8),
    }
}

/// CheckOptimizedMovements reads the stdin termios and reports whether hard
/// tabs (TABDLY==TAB0) and backspace (BSDLY==BS0) optimizations are enabled.
fn check_optimized_movements() -> (bool, bool) {
    use std::os::fd::AsRawFd;
    #[cfg(unix)]
    {
        let fd = std::io::stdin().as_raw_fd();
        let mut t: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(fd, &mut t) } != 0 {
            return (false, false);
        }
        let hard_tabs = t.c_oflag & libc::TABDLY == libc::TAB0;
        #[cfg(target_os = "macos")]
        let backspace = t.c_lflag & libc::BSDLY == libc::BS0;
        #[cfg(not(target_os = "macos"))]
        let backspace = false;
        (hard_tabs, backspace)
    }
    #[cfg(not(unix))]
    {
        let _ = ();
        (true, true)
    }
}
