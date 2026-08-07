//! Cleanroom Rust port of upstream Go source file: `options.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Options
//!
//! Options for configuring Program behavior on initialization.
//! </public-docs>

use crate::nil_renderer::NilRenderer;
use crate::program::Program;
use crate::standard_renderer::StandardRenderer;
use crate::model::{Model, Msg};

/// <upstream-comment>
/// ProgramOption is used to set options when initializing a Program.
/// </upstream-comment>
pub type ProgramOption<M> = Box<dyn FnOnce(&mut Program<M>)>;

/// <upstream-comment>
/// WithAltScreen starts the program with alternate screen buffer enabled.
/// </upstream-comment>
pub fn with_alt_screen<M: Model>() -> ProgramOption<M> {
    Box::new(|p: &mut Program<M>| {
        p.renderer.enter_alt_screen();
    })
}

/// <upstream-comment>
/// WithoutRenderer disables the renderer.
/// </upstream-comment>
pub fn without_renderer<M: Model>() -> ProgramOption<M> {
    Box::new(|p: &mut Program<M>| {
        p.renderer = Box::new(NilRenderer);
    })
}

/// <upstream-comment>
/// WithFPS sets custom maximum FPS for renderer.
/// </upstream-comment>
pub fn with_fps<M: Model>(fps: u32) -> ProgramOption<M> {
    Box::new(move |p: &mut Program<M>| {
        p.renderer = Box::new(StandardRenderer::new(fps));
    })
}

/// <upstream-comment>
/// WithFilter supplies an event filter that can modify or intercept messages before processing.
/// </upstream-comment>
pub fn with_filter<M: Model, F>(filter: F) -> ProgramOption<M>
where
    F: Fn(&M, Box<dyn Msg>) -> Option<Box<dyn Msg>> + Send + Sync + 'static,
{
    Box::new(move |p: &mut Program<M>| {
        p.filter = Some(Box::new(filter));
    })
}
