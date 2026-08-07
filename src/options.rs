//! Cleanroom Rust port of upstream Go source file: `options.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Program Options
//!
//! Program options (`with_fps`, `without_renderer`, `with_filter`, `with_window_size`, `without_signals`).
//! </public-docs>

use crate::model::{Model, Msg};

/// Program configuration options.
pub struct ProgramOptions<M: Model> {
    /// Target FPS framerate limit.
    pub fps: u32,
    /// Disable renderer (for daemon or headless usage).
    pub disable_renderer: bool,
    /// Disable OS signal handling.
    pub disable_signals: bool,
    /// Initial terminal width override.
    pub width: usize,
    /// Initial terminal height override.
    pub height: usize,
    /// Optional event filter.
    pub filter: Option<Box<dyn Fn(&M, Box<dyn Msg>) -> Option<Box<dyn Msg>> + Send + Sync>>,
}

impl<M: Model> Default for ProgramOptions<M> {
    fn default() -> Self {
        Self {
            fps: 60,
            disable_renderer: false,
            disable_signals: false,
            width: 0,
            height: 0,
            filter: None,
        }
    }
}
