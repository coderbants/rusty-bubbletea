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
//!
//! <user-docs>
//! [`Program`] runs a model's event loop. Use [`Program::handle`] when the
//! program must be run on another thread and controlled from the outside.
//! [`ProgramOptions`](crate::options::ProgramOptions) supplies deterministic
//! input, output, terminal-size, renderer, and cancellation behavior.
//! </user-docs>
//!
//! Maintainer note: setup, event dispatch, effect execution, and renderer
//! shutdown are deliberately kept as separate phases. Shared lifecycle state
//! makes pre-start commands, external cancellation, and final shutdown
//! observable without borrowing the model across threads.

use std::fmt;
use std::io::{IsTerminal, Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};

/// Message channel used to feed the program's event loop.
type MsgChannel = Sender<Box<dyn Msg>>;
/// Message channel used to receive messages into the program's event loop.
type MsgReceiver = Receiver<Box<dyn Msg>>;
use std::sync::{Arc, Mutex, MutexGuard};
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
use crate::nil_renderer::NilRenderer;
use crate::options::ProgramOptions;
use crate::paste::PasteMsg;
use crate::profile::ColorProfileMsg;
use crate::raw::RawMsg;
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

/// Error text returned when the program cannot safely change or restore the
/// terminal mode.
pub const ERR_TERMINAL: &str = "terminal mode operation failed";

/// ProgramError is the error type returned by [Program::run].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramError {
    /// The program was killed (context cancellation or external kill).
    Killed,
    /// The program was interrupted (SIGINT or [InterruptMsg]).
    Interrupted,
    /// The program recovered from a panic.
    Panic,
    /// A terminal mode transition or restoration failed.
    Terminal,
}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramError::Killed => write!(f, "{}", ERR_PROGRAM_KILLED),
            ProgramError::Interrupted => write!(f, "{}", ERR_INTERRUPTED),
            ProgramError::Panic => write!(f, "{}", ERR_PROGRAM_PANIC),
            ProgramError::Terminal => write!(f, "{}", ERR_TERMINAL),
        }
    }
}

impl std::error::Error for ProgramError {}

const PROGRAM_NEW: u8 = 0;
const PROGRAM_RUNNING: u8 = 1;
const PROGRAM_FINISHED: u8 = 2;

/// A cloneable control surface for a running [`Program`].
///
/// A handle may be created before the program is moved to its runner thread.
/// Messages sent before startup remain ordered in the program's event queue;
/// messages sent after shutdown are ignored. `kill` requests an error
/// shutdown, while `quit` requests the normal graceful shutdown.
#[derive(Clone)]
pub struct ProgramHandle {
    msg_tx: MsgChannel,
    state: Arc<AtomicU8>,
    killed: Arc<AtomicBool>,
}

impl ProgramHandle {
    /// Sends a typed message to the program event loop.
    ///
    /// The message is ignored after the program has finished or after a kill
    /// request. This method never blocks on the model or renderer.
    pub fn send(&self, msg: Box<dyn Msg>) {
        if self.killed.load(Ordering::SeqCst)
            || self.state.load(Ordering::SeqCst) == PROGRAM_FINISHED
        {
            return;
        }
        let _ = self.msg_tx.send(msg);
    }

    /// Requests a graceful program shutdown.
    pub fn quit(&self) {
        self.send(Box::new(QuitMsg));
    }

    /// Requests an immediate program shutdown with [`ProgramError::Killed`].
    pub fn kill(&self) {
        if self.state.load(Ordering::SeqCst) == PROGRAM_FINISHED {
            return;
        }
        self.killed.store(true, Ordering::SeqCst);
        let _ = self.msg_tx.send(Box::new(QuitMsg));
    }

    /// Waits until renderer and terminal cleanup has completed.
    ///
    /// Call this after starting the program. A handle intentionally does not
    /// guess whether a program that has not been started will be started later.
    pub fn wait(&self) {
        while self.state.load(Ordering::SeqCst) != PROGRAM_FINISHED {
            thread::sleep(Duration::from_millis(10));
        }
    }
}

/// Program is the runner for a Bubble Tea v2.0.8 application.
pub struct Program<M: Model> {
    model: M,
    options: ProgramOptions<M>,
    renderer: Option<Arc<Mutex<Box<dyn Renderer>>>>,
    msg_tx: MsgChannel,
    msg_rx: Option<MsgReceiver>,
    state: Arc<AtomicU8>,
    killed: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
    render_thread: Option<thread::JoinHandle<()>>,
}

impl<M: Model> Program<M> {
    /// Creates a new program for the given model with default options.
    ///
    /// The returned program owns its event queue immediately, so a
    /// [`ProgramHandle`] can enqueue startup messages before [`Self::run`]
    /// takes ownership of the runner.
    pub fn new(model: M) -> Self {
        let (msg_tx, msg_rx) = channel();
        Self {
            model,
            options: ProgramOptions::default(),
            renderer: None,
            msg_tx,
            msg_rx: Some(msg_rx),
            state: Arc::new(AtomicU8::new(PROGRAM_NEW)),
            killed: Arc::new(AtomicBool::new(false)),
            stopping: Arc::new(AtomicBool::new(false)),
            render_thread: None,
        }
    }

    /// Sets custom program options.
    pub fn with_options(mut self, options: ProgramOptions<M>) -> Self {
        self.options = options;
        self
    }

    /// Returns a cloneable control surface for this program.
    ///
    /// ```
    /// # use rusty_bubbletea::{Cmd, Model, Msg, Program, View};
    /// # struct Example;
    /// # impl Model for Example {
    /// #     fn update(&mut self, _msg: &dyn Msg) -> Cmd { None }
    /// #     fn view(&self) -> View { View::new("") }
    /// # }
    /// let program = Program::new(Example);
    /// let handle = program.handle();
    /// handle.quit();
    /// ```
    pub fn handle(&self) -> ProgramHandle {
        ProgramHandle {
            msg_tx: self.msg_tx.clone(),
            state: self.state.clone(),
            killed: self.killed.clone(),
        }
    }

    /// <upstream-comment>Send sends a message to the main update function, effectively allowing
    /// messages to be injected from outside the program for interoperability
    /// purposes.</upstream-comment>
    pub fn send(&self, msg: Box<dyn Msg>) {
        self.handle().send(msg);
    }

    /// <upstream-comment>Quit is a convenience function for quitting Bubble Tea programs. Use it
    /// when you need to shut down a Bubble Tea program from the outside.</upstream-comment>
    pub fn quit(&self) {
        self.handle().quit();
    }

    /// <upstream-comment>Kill stops the program immediately and restores the former terminal state.
    /// The final render that you would normally see when quitting will be skipped.
    /// [Program.Run] returns a [ErrProgramKilled] error.</upstream-comment>
    pub fn kill(&self) {
        self.handle().kill();
    }

    /// <upstream-comment>Wait waits/blocks until the underlying Program finished shutting down.</upstream-comment>
    pub fn wait(&self) {
        self.handle().wait();
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

    fn renderer_guard(&self) -> Option<MutexGuard<'_, Box<dyn Renderer>>> {
        self.renderer.as_ref()?.lock().ok()
    }

    fn render_view(&self, view: crate::view::View) {
        if let Some(mut renderer) = self.renderer_guard() {
            renderer.render(view);
        }
    }

    fn write_direct(&self, text: &str) {
        if let Some(mut renderer) = self.renderer_guard() {
            let _ = renderer.write_direct(text);
        }
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
            // Restore the terminal before suspension. A failed restoration is
            // fatal because continuing could leave the user's TTY in raw mode.
            disable_raw_mode().map_err(|_| ProgramError::Terminal)?;
            self.send_resume_later(tx);
        }

        if processed_msg.as_ref().as_any().is::<ClearScreenMsg>() {
            if let Some(mut renderer) = self.renderer_guard() {
                renderer.clear_screen();
            }
        } else if processed_msg
            .as_ref()
            .as_any()
            .is::<crate::color::RequestBackgroundColorMsg>()
        {
            self.write_direct(rusty_x_ansi::background::REQUEST_BACKGROUND_COLOR);
        } else if processed_msg
            .as_ref()
            .as_any()
            .is::<crate::color::RequestForegroundColorMsg>()
        {
            self.write_direct(rusty_x_ansi::background::REQUEST_FOREGROUND_COLOR);
        } else if processed_msg
            .as_ref()
            .as_any()
            .is::<crate::color::RequestCursorColorMsg>()
        {
            self.write_direct(rusty_x_ansi::background::REQUEST_CURSOR_COLOR);
        } else if let Some(cap) = processed_msg
            .as_ref()
            .as_any()
            .downcast_ref::<crate::termcap::RequestCapabilityMsg>()
        {
            // Mirror upstream `p.execute(ansi.RequestTermcap(cap))`: write the
            // XTGETTCAP query (DCS + q <Pt> ST) to the configured output.
            let mut seq = String::from("\x1bP+q");
            for byte in cap.0.as_bytes() {
                seq.push_str(&format!("{byte:02X}"));
            }
            seq.push_str("\x1b\\");
            self.write_direct(&seq);
            return Ok(false);
        } else if processed_msg
            .as_ref()
            .as_any()
            .is::<crate::xterm::RequestTerminalVersionMsg>()
        {
            // Mirror upstream `p.execute(ansi.RequestNameVersion)` using the
            // configured renderer output rather than process-global stdout.
            self.write_direct("\x1b[>q");
            return Ok(false);
        } else if processed_msg.as_ref().as_any().is::<RequestWindowSizeMsg>() {
            let (width, height) = configured_window_size(&self.options);
            let _ = tx.send(Box::new(WindowSizeMsg { width, height }));
            // RequestWindowSizeMsg itself is internal — don't pass to model.update.
            return Ok(false);
        } else if let Some(ws) = processed_msg
            .as_ref()
            .as_any()
            .downcast_ref::<WindowSizeMsg>()
        {
            // Resize the renderer first, then fall through to model.update below.
            if let Some(mut renderer) = self.renderer_guard() {
                renderer.resize(ws.width, ws.height);
            }
        } else if let Some(exec_msg) = processed_msg.as_ref().as_any().downcast_ref::<ExecMsg>() {
            disable_raw_mode().map_err(|_| ProgramError::Terminal)?;
            let mut command = std::process::Command::new(&exec_msg.cmd);
            command.args(&exec_msg.args);
            let _ = command.status();
            enable_raw_mode().map_err(|_| ProgramError::Terminal)?;
        } else if let Some(print_msg) = processed_msg
            .as_ref()
            .as_any()
            .downcast_ref::<PrintLineMsg>()
        {
            // Insert the line above the TUI without routing through model.update.
            if let Some(mut renderer) = self.renderer_guard() {
                let _ = renderer.insert_above(print_msg.message_body.clone());
                renderer.render(self.model.view());
            }
            return Ok(false);
        } else if processed_msg.as_ref().as_any().is::<RawMsg>() {
            if let Some(raw) = processed_msg.as_ref().as_any().downcast_ref::<RawMsg>() {
                self.write_direct(&raw.0);
            }
            return Ok(false);
        } else if let Some(_env) = processed_msg.as_ref().as_any().downcast_ref::<EnvMsg>() {
            // EnvMsg remains visible to the model below, matching ordinary
            // Bubble Tea startup messages.
        } else if let Some(profile) = processed_msg
            .as_ref()
            .as_any()
            .downcast_ref::<ColorProfileMsg>()
        {
            if let Some(mut renderer) = self.renderer_guard() {
                renderer.set_color_profile(color_profile(profile.profile));
            }
        } else if processed_msg.as_ref().as_any().is::<ResumeMsg>() {
            enable_raw_mode().map_err(|_| ProgramError::Terminal)?;
        }

        // Dispatch BatchMsg and SequenceMsg recursively. Batch commands run
        // concurrently, while sequence commands preserve source order.
        if processed_msg.as_ref().as_any().is::<BatchMsg>() {
            let any = processed_msg.into_any();
            if let Ok(batch) = any.downcast::<BatchMsg>() {
                let tx_clone = tx.clone();
                thread::spawn(move || exec_batch_msg(*batch, &tx_clone));
            }
            return Ok(false);
        }

        // Route mouse messages through the renderer's optional interceptor;
        // the original event still falls through to model.update.
        let mouse_msg = {
            let any = processed_msg.as_ref().as_any();
            if let Some(mouse) = any.downcast_ref::<MouseClickMsg>() {
                Some(crate::mouse::MouseMsg::Click(mouse.clone()))
            } else if let Some(mouse) = any.downcast_ref::<MouseMotionMsg>() {
                Some(crate::mouse::MouseMsg::Motion(mouse.clone()))
            } else if let Some(mouse) = any.downcast_ref::<MouseReleaseMsg>() {
                Some(crate::mouse::MouseMsg::Release(mouse.clone()))
            } else {
                any.downcast_ref::<MouseWheelMsg>()
                    .map(|mouse| crate::mouse::MouseMsg::Wheel(mouse.clone()))
            }
        };
        if let Some(mouse_msg) = mouse_msg {
            let command = self
                .renderer_guard()
                .and_then(|mut renderer| renderer.on_mouse(mouse_msg));
            if let Some(command) = command {
                let tx_clone = tx.clone();
                thread::spawn(move || {
                    if let Some(new_msg) = command() {
                        let _ = tx_clone.send(new_msg);
                    }
                });
            }
        }

        if processed_msg.as_ref().as_any().is::<SequenceMsg>() {
            let any = processed_msg.into_any();
            if let Ok(sequence) = any.downcast::<SequenceMsg>() {
                let tx_clone = tx.clone();
                thread::spawn(move || exec_sequence_msg(*sequence, &tx_clone));
            }
            return Ok(false);
        }

        let command = self.model.update(&*processed_msg);
        self.render_view(self.model.view());

        if let Some(command) = command {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                if let Some(new_msg) = command() {
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

    fn stop_render_thread(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        if let Some(render_thread) = self.render_thread.take() {
            let _ = render_thread.join();
        }
    }

    fn cleanup_renderer(&mut self, graceful: bool) -> std::io::Result<()> {
        self.stop_render_thread();
        if let Some(renderer) = self.renderer.take() {
            if let Ok(mut renderer) = renderer.lock() {
                if graceful {
                    renderer.render(self.model.view());
                    let _ = renderer.flush(true);
                }
                let _ = renderer.close();
            }
        }
        disable_raw_mode()
    }

    fn run_inner(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let rx = self.msg_rx.take().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "program event loop has already been consumed",
            )
        })?;
        let tx = self.msg_tx.clone();

        let env_pairs = self
            .options
            .environ
            .take()
            .unwrap_or_else(|| std::env::vars().collect());
        let env_strings: Vec<String> = env_pairs
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect();
        let term = env_value(&env_strings, "TERM")
            .filter(|value| !value.is_empty())
            .unwrap_or("xterm-256color")
            .to_owned();
        let (width, height) = configured_window_size(&self.options);
        let output_is_stdout = self.options.output.is_none();
        let output: Box<dyn Write + Send + Sync> = match self.options.output.take() {
            Some(output) => output,
            None => Box::new(std::io::stdout()),
        };
        let renderer: Box<dyn Renderer> = if self.options.disable_renderer {
            Box::new(NilRenderer)
        } else {
            Box::new(crate::cursed_renderer::new_cursed_renderer(
                output,
                &env_strings,
                width,
                height,
            ))
        };
        self.renderer = Some(Arc::new(Mutex::new(renderer)));

        let profile = match self.options.color_profile {
            Some(profile) => color_profile(profile),
            None => detect_color_profile(&env_strings, output_is_stdout),
        };
        let (hard_tabs, backspace) = if self.options.disable_renderer {
            (false, false)
        } else {
            check_optimized_movements()
        };
        let renderer_ready = {
            if let Some(mut renderer) = self.renderer_guard() {
                renderer.start();
                renderer.set_color_profile(profile);
                renderer.set_optimizations(hard_tabs, backspace, false);
                true
            } else {
                false
            }
        };
        if !renderer_ready {
            self.cleanup_renderer(false)?;
            return Err(Box::new(std::io::Error::other(
                "program renderer lock is unavailable",
            )));
        }

        let input_disabled = self.options.input_disabled();
        let use_raw_mode = !self.options.disable_renderer
            && !input_disabled
            && (self.options.input.is_some() || std::io::stdin().is_terminal());
        if use_raw_mode {
            if let Err(enable_error) = enable_raw_mode() {
                return match self.cleanup_renderer(false) {
                    Ok(()) => Err(Box::new(enable_error)),
                    Err(cleanup_error) => Err(Box::new(cleanup_error)),
                };
            }
        }

        if !input_disabled {
            let reader = self.options.input.take();
            let input_tx = tx.clone();
            thread::spawn(move || {
                let reader: Box<dyn Read + Send> = match reader {
                    Some(reader) => reader,
                    None => Box::new(std::io::stdin()),
                };
                let mut terminal_reader =
                    rusty_ultraviolet::terminal_reader::new_terminal_reader(reader, &term);
                terminal_reader.set_legacy(rusty_ultraviolet::LegacyKeyEncoding::default());
                let (decoded_tx, decoded_rx) =
                    std::sync::mpsc::channel::<rusty_ultraviolet::DecodedEvent>();
                let streamer = thread::spawn(move || {
                    let _ = terminal_reader.stream_events(&decoded_tx);
                });
                for event in decoded_rx {
                    if let Some(msg) = decoded_to_msg(event) {
                        if input_tx.send(msg).is_err() {
                            break;
                        }
                    }
                }
                let _ = streamer.join();
            });
        }

        if let Some(cmd) = self.model.init() {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                if let Some(msg) = cmd() {
                    let _ = tx_clone.send(msg);
                }
            });
        }

        let _ = tx.send(Box::new(WindowSizeMsg { width, height }));
        let _ = tx.send(Box::new(EnvMsg::new(env_pairs)));
        let msg_profile = match profile {
            rusty_colorprofile::Profile::TrueColor => crate::profile::ColorProfile::TrueColor,
            rusty_colorprofile::Profile::Ansi256 => crate::profile::ColorProfile::ANSI256,
            rusty_colorprofile::Profile::Ansi => crate::profile::ColorProfile::ANSI,
            rusty_colorprofile::Profile::Ascii
            | rusty_colorprofile::Profile::NoTty
            | rusty_colorprofile::Profile::Unknown => crate::profile::ColorProfile::Ascii,
        };
        let _ = tx.send(Box::new(ColorProfileMsg {
            profile: msg_profile,
        }));

        if !self.options.disable_renderer && should_query_synchronized_output(&env_strings) {
            self.write_direct("\x1b[?2026$p\x1b[?2027$p");
        }

        self.render_view(self.model.view());

        let ticker_renderer = self.renderer.as_ref().cloned();
        let stopping = self.stopping.clone();
        let fps = self.options.fps.clamp(1, 120);
        self.render_thread = ticker_renderer.map(|renderer| {
            thread::spawn(move || {
                let interval = Duration::from_millis(1000 / fps as u64);
                while !stopping.load(Ordering::SeqCst) {
                    thread::sleep(interval);
                    if stopping.load(Ordering::SeqCst) {
                        break;
                    }
                    if let Ok(mut renderer) = renderer.lock() {
                        let _ = renderer.flush(false);
                    }
                }
            })
        });

        let external_ctx = self.options.context.clone();
        let result = loop {
            if self.killed.load(Ordering::SeqCst) {
                break Err(ProgramError::Killed);
            }
            if let Some(ctx) = &external_ctx {
                if ctx.done() {
                    break Err(ProgramError::Killed);
                }
            }
            match rx.recv_timeout(Duration::from_millis(10)) {
                Ok(msg) => match self.handle_msg(msg, &tx) {
                    Ok(true) => break Ok(()),
                    Ok(false) => continue,
                    Err(error) => break Err(error),
                },
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break Ok(()),
            }
        };

        let graceful = result.is_ok();
        self.cleanup_renderer(graceful)?;
        result.map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
    }

    /// Runs the Bubble Tea v2.0.8 event loop until quit.
    pub fn run(mut self) -> Result<M, Box<dyn std::error::Error>> {
        if self
            .state
            .compare_exchange(
                PROGRAM_NEW,
                PROGRAM_RUNNING,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_err()
        {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "program can only be run once",
            )));
        }

        let result = if self.options.disable_catch_panics {
            self.run_inner()
        } else {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.run_inner())) {
                Ok(result) => result,
                Err(_) => match self.cleanup_renderer(false) {
                    Ok(()) => Err(Box::new(ProgramError::Panic) as Box<dyn std::error::Error>),
                    Err(error) => Err(Box::new(error) as Box<dyn std::error::Error>),
                },
            }
        };
        self.state.store(PROGRAM_FINISHED, Ordering::SeqCst);

        match result {
            Ok(()) => Ok(self.model),
            Err(error) => Err(error),
        }
    }
}

/// Returns the configured initial terminal dimensions, with a stable fallback
/// for headless or non-terminal execution.
fn configured_window_size<M: Model>(options: &ProgramOptions<M>) -> (usize, usize) {
    let detected = match term_size() {
        Ok((width, height)) => (width as usize, height as usize),
        Err(_) => (80, 24),
    };
    let width = if options.width == 0 {
        detected.0
    } else {
        options.width
    };
    let height = if options.height == 0 {
        detected.1
    } else {
        options.height
    };
    (width.max(1), height.max(1))
}

/// Looks up one `KEY=VALUE` entry from the configured environment snapshot.
fn env_value<'a>(env: &'a [String], key: &str) -> Option<&'a str> {
    env.iter().find_map(|entry| {
        let (entry_key, value) = entry.split_once('=')?;
        (entry_key == key).then_some(value)
    })
}

/// Maps the public Bubble Tea color profile to the renderer's profile type.
fn color_profile(profile: crate::profile::ColorProfile) -> rusty_colorprofile::Profile {
    match profile {
        crate::profile::ColorProfile::TrueColor => rusty_colorprofile::Profile::TrueColor,
        crate::profile::ColorProfile::ANSI256 => rusty_colorprofile::Profile::Ansi256,
        crate::profile::ColorProfile::ANSI => rusty_colorprofile::Profile::Ansi,
        crate::profile::ColorProfile::Ascii => rusty_colorprofile::Profile::Ascii,
    }
}

/// Detects the renderer color profile using the configured environment and,
/// when stdout is the configured output, the process stdout terminal handle.
fn detect_color_profile(env: &[String], output_is_stdout: bool) -> rusty_colorprofile::Profile {
    let output_fd = if output_is_stdout {
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            Some(std::io::stdout().as_raw_fd())
        }
        #[cfg(not(unix))]
        {
            None
        }
    } else {
        None
    };
    rusty_ultraviolet::terminal_screen::detect_color_profile(
        output_fd,
        &rusty_ultraviolet::Environ(env.to_vec()),
    )
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
fn should_query_synchronized_output(env: &[String]) -> bool {
    let term_type = env_value(env, "TERM").unwrap_or_default();
    let term_prog = env_value(env, "TERM_PROGRAM");
    let ssh_tty = env_value(env, "SSH_TTY").is_some();
    let wt_session = env_value(env, "WT_SESSION").is_some();

    let ok_term_prog = term_prog.is_some();
    wt_session
        || term_type.contains("ghostty")
        || term_type.contains("wezterm")
        || (!ok_term_prog && !ssh_tty)
        || (!ssh_tty && !term_prog.unwrap_or("").contains("Apple"))
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
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;

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

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_ultraviolet as uv;

    #[test]
    fn test_decoded_to_msg_events() {
        let k = uv::Key {
            text: "a".to_string(),
            mod_: uv::KeyMod(0),
            code: 'a' as u32,
            shifted_code: 0,
            base_code: 0,
            is_repeat: false,
        };
        let ev = uv::DecodedEvent::KeyPress(k.clone());
        let msg = decoded_to_msg(ev).unwrap();
        assert!(msg.as_ref().as_any().is::<KeyPressMsg>());

        let ev_rel = uv::DecodedEvent::KeyRelease(k);
        let msg_rel = decoded_to_msg(ev_rel).unwrap();
        assert!(msg_rel.as_ref().as_any().is::<KeyReleaseMsg>());

        let m = uv::Mouse {
            x: 10,
            y: 20,
            button: uv::MOUSE_LEFT,
            mod_: uv::KeyMod(0),
        };
        assert!(decoded_to_msg(uv::DecodedEvent::MouseClick(m))
            .unwrap()
            .as_ref()
            .as_any()
            .is::<MouseClickMsg>());
        assert!(decoded_to_msg(uv::DecodedEvent::MouseRelease(m))
            .unwrap()
            .as_ref()
            .as_any()
            .is::<MouseReleaseMsg>());
        assert!(decoded_to_msg(uv::DecodedEvent::MouseWheel(m))
            .unwrap()
            .as_ref()
            .as_any()
            .is::<MouseWheelMsg>());
        assert!(decoded_to_msg(uv::DecodedEvent::MouseMotion(m))
            .unwrap()
            .as_ref()
            .as_any()
            .is::<MouseMotionMsg>());

        assert!(decoded_to_msg(uv::DecodedEvent::WindowSize(uv::Size {
            width: 80,
            height: 24
        }))
        .unwrap()
        .as_ref()
        .as_any()
        .is::<WindowSizeMsg>());
        assert!(decoded_to_msg(uv::DecodedEvent::Paste("pasted".into()))
            .unwrap()
            .as_ref()
            .as_any()
            .is::<PasteMsg>());
        assert!(decoded_to_msg(uv::DecodedEvent::Focus)
            .unwrap()
            .as_ref()
            .as_any()
            .is::<FocusMsg>());
        assert!(decoded_to_msg(uv::DecodedEvent::Blur)
            .unwrap()
            .as_ref()
            .as_any()
            .is::<BlurMsg>());
        assert!(decoded_to_msg(uv::DecodedEvent::KeyboardEnhancements(1))
            .unwrap()
            .as_ref()
            .as_any()
            .is::<KeyboardEnhancementsMsg>());
        assert!(
            decoded_to_msg(uv::DecodedEvent::CursorPosition { x: 5, y: 6 })
                .unwrap()
                .as_ref()
                .as_any()
                .is::<CursorPositionMsg>()
        );
        assert!(decoded_to_msg(uv::DecodedEvent::Capability("cap".into()))
            .unwrap()
            .as_ref()
            .as_any()
            .is::<crate::termcap::CapabilityMsg>());
        assert!(
            decoded_to_msg(uv::DecodedEvent::TerminalVersion("v1".into()))
                .unwrap()
                .as_ref()
                .as_any()
                .is::<crate::xterm::TerminalVersionMsg>()
        );
    }

    #[test]
    fn test_program_error_display() {
        assert_eq!(format!("{}", ProgramError::Killed), ERR_PROGRAM_KILLED);
        assert_eq!(format!("{}", ProgramError::Interrupted), ERR_INTERRUPTED);
        assert_eq!(format!("{}", ProgramError::Panic), ERR_PROGRAM_PANIC);
        assert_eq!(format!("{}", ProgramError::Terminal), ERR_TERMINAL);
    }
}
