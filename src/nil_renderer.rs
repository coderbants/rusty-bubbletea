//! Cleanroom Rust port of upstream Go source file: `nil_renderer.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Nil Renderer
//!
//! NilRenderer is a no-op implementation of Renderer for headless or testing environments.
//! </public-docs>

use crate::renderer::Renderer;

/// <upstream-comment>
/// NilRenderer is a no-op renderer implementation.
/// </upstream-comment>
#[derive(Debug, Default, Clone, Copy)]
pub struct NilRenderer;

impl Renderer for NilRenderer {
    fn start(&mut self) {}
    fn stop(&mut self) {}
    fn kill(&mut self) {}
    fn write(&mut self, _s: String) {}
    fn repaint(&mut self) {}
    fn clear_screen(&mut self) {}
    fn alt_screen(&self) -> bool {
        false
    }
    fn enter_alt_screen(&mut self) {}
    fn exit_alt_screen(&mut self) {}
    fn show_cursor(&mut self) {}
    fn hide_cursor(&mut self) {}
    fn enable_mouse_cell_motion(&mut self) {}
    fn disable_mouse_cell_motion(&mut self) {}
    fn enable_mouse_all_motion(&mut self) {}
    fn disable_mouse_all_motion(&mut self) {}
    fn enable_mouse_sgr_mode(&mut self) {}
    fn disable_mouse_sgr_mode(&mut self) {}
    fn enable_bracketed_paste(&mut self) {}
    fn disable_bracketed_paste(&mut self) {}
    fn bracketed_paste_active(&self) -> bool {
        false
    }
    fn set_window_title(&mut self, _title: &str) {}
    fn report_focus(&self) -> bool {
        false
    }
    fn enable_report_focus(&mut self) {}
    fn disable_report_focus(&mut self) {}
}
