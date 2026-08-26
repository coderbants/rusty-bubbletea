//! Cleanroom Rust port of upstream Go source file: `options.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <user-docs>
//! # Program Options
//!
//! Program options (`with_fps`, `without_renderer`, `with_filter`, `with_window_size`,
//! `with_context`, `with_output`, `with_input`, `with_environment`,
//! `without_signal_handler`, `without_catch_panics`, `without_signals`, `with_color_profile`).
//! </user-docs>
//!
//! Maintainer note: options are consumed once by [`crate::program::Program::run`].
//! The input sentinel keeps the default stdin behavior distinct from an explicit
//! `with_input(None)`, which disables input for deterministic headless programs.

use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::model::{Model, Msg};
use crate::profile::ColorProfile;

/// A cancellation context mirroring `context.Context` for the program lifecycle.
#[derive(Debug, Clone, Default)]
pub struct Context {
    cancelled: Arc<AtomicBool>,
}

impl Context {
    /// Returns a new cancellable context.
    pub fn new() -> Context {
        Context {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Done returns whether the context has been cancelled.
    pub fn done(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Cancel cancels the context.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

/// An event filter invoked before the program processes a message, mirroring
/// the function type of `WithFilter`.
pub type EventFilter<M> = Box<dyn Fn(&M, Box<dyn Msg>) -> Option<Box<dyn Msg>> + Send + Sync>;

/// Program configuration options.
pub struct ProgramOptions<M: Model> {
    /// Target FPS framerate limit.
    pub fps: u32,
    /// Disable renderer (for daemon or headless usage).
    pub disable_renderer: bool,
    /// Disable OS signal handling.
    pub disable_signals: bool,
    /// Disable the signal handler that Bubble Tea sets up for programs.
    pub disable_signal_handler: bool,
    /// Disable the panic catching that Bubble Tea does by default.
    pub disable_catch_panics: bool,
    /// Initial terminal width override.
    pub width: usize,
    /// Initial terminal height override.
    pub height: usize,
    /// Optional event filter.
    pub filter: Option<EventFilter<M>>,
    /// Input reader override. The default uses stdin; [`Self::with_input`] with
    /// `None` disables input entirely.
    pub input: Option<Box<dyn Read + Send + Sync>>,
    /// Distinguishes the default stdin input from an explicit disabled input.
    input_disabled: bool,
    /// Output writer override; None means stdout.
    pub output: Option<Box<dyn Write + Send + Sync>>,
    /// Environment variables used by the program.
    pub environ: Option<Vec<(String, String)>>,
    /// Forced color profile.
    pub color_profile: Option<ColorProfile>,
    /// External cancellation context.
    pub context: Option<Context>,
}

impl<M: Model> Default for ProgramOptions<M> {
    fn default() -> Self {
        Self {
            fps: 60,
            disable_renderer: false,
            disable_signals: false,
            disable_signal_handler: false,
            disable_catch_panics: false,
            width: 0,
            height: 0,
            filter: None,
            input: None,
            input_disabled: false,
            output: None,
            environ: None,
            color_profile: None,
            context: None,
        }
    }
}

impl<M: Model> ProgramOptions<M> {
    /// <upstream-comment>WithContext lets you specify a context in which to run the Program. This is
    /// useful if you want to cancel the execution from outside. When a Program gets
    /// cancelled it will exit with an error ErrProgramKilled.</upstream-comment>
    pub fn with_context(mut self, ctx: Context) -> Self {
        self.context = Some(ctx);
        self
    }

    /// <upstream-comment>WithOutput sets the output which, by default, is stdout. In most cases you
    /// won't need to use this.</upstream-comment>
    pub fn with_output(mut self, output: Box<dyn Write + Send + Sync>) -> Self {
        self.output = Some(output);
        self
    }

    /// <upstream-comment>WithInput sets the input which, by default, is stdin. In most cases you
    /// won't need to use this. To disable input entirely pass None.</upstream-comment>
    pub fn with_input(mut self, input: Option<Box<dyn Read + Send + Sync>>) -> Self {
        self.input_disabled = input.is_none();
        self.input = input;
        self
    }

    /// Returns whether input was explicitly disabled with [`Self::with_input`].
    pub(crate) fn input_disabled(&self) -> bool {
        self.input_disabled
    }

    /// <upstream-comment>WithEnvironment sets the environment variables that the program will use.
    /// This is useful when the program is running in a remote session (e.g. SSH) and
    /// you want to pass the environment variables from the remote session to the
    /// program.</upstream-comment>
    pub fn with_environment(mut self, env: Vec<(String, String)>) -> Self {
        self.environ = Some(env);
        self
    }

    /// <upstream-comment>WithoutSignalHandler disables the signal handler that Bubble Tea sets up for
    /// Programs. This is useful if you want to handle signals yourself.</upstream-comment>
    pub fn without_signal_handler(mut self) -> Self {
        self.disable_signal_handler = true;
        self
    }

    /// <upstream-comment>WithoutCatchPanics disables the panic catching that Bubble Tea does by
    /// default. If panic catching is disabled the terminal will be in a fairly
    /// unusable state after a panic because Bubble Tea will not perform its usual
    /// cleanup on exit.</upstream-comment>
    pub fn without_catch_panics(mut self) -> Self {
        self.disable_catch_panics = true;
        self
    }

    /// <upstream-comment>WithoutSignals will ignore OS signals.
    /// This is mainly useful for testing.</upstream-comment>
    pub fn without_signals(mut self) -> Self {
        self.disable_signals = true;
        self
    }

    /// <upstream-comment>WithoutRenderer disables the renderer. When this is set output and log
    /// statements will be plainly sent to stdout (or another output if one is set)
    /// without any rendering and redrawing logic.</upstream-comment>
    pub fn without_renderer(mut self) -> Self {
        self.disable_renderer = true;
        self
    }

    /// <upstream-comment>WithFilter supplies an event filter that will be invoked before Bubble Tea
    /// processes a tea.Msg. The event filter can return any tea.Msg which will then
    /// get handled by Bubble Tea instead of the original event. If the event filter
    /// returns None, the event will be ignored and Bubble Tea will not process it.</upstream-comment>
    pub fn with_filter(mut self, filter: EventFilter<M>) -> Self {
        self.filter = Some(filter);
        self
    }

    /// <upstream-comment>WithFPS sets a custom maximum FPS at which the renderer should run. If
    /// less than 1, the default value of 60 will be used. If over 120, the FPS
    /// will be capped at 120.</upstream-comment>
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = if fps == 0 { 60 } else { fps.min(120) };
        self
    }

    /// <upstream-comment>WithColorProfile sets the color profile that the program will use.</upstream-comment>
    pub fn with_color_profile(mut self, profile: ColorProfile) -> Self {
        self.color_profile = Some(profile);
        self
    }

    /// <upstream-comment>WithWindowSize sets the initial size of the terminal window. This is useful
    /// when you need to set the initial size of the terminal window, for example
    /// during testing or when you want to run your program in a non-interactive
    /// environment.</upstream-comment>
    pub fn with_window_size(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}
