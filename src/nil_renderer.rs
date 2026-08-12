//! Cleanroom Rust port of upstream Go source file: `nil_renderer.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Nil Renderer
//!
//! No-op implementation of `Renderer` trait for headless testing in Bubble Tea v2.0.8.
//! </public-docs>

use crate::model::Cmd;
use crate::mouse::MouseMsg;
use crate::renderer::Renderer;
use crate::view::View;

/// NilRenderer is a no-op renderer implementation.
#[derive(Debug, Default, Clone, Copy)]
pub struct NilRenderer;

impl Renderer for NilRenderer {
    fn start(&mut self) {}
    fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn render(&mut self, _view: View) {}
    fn flush(&mut self, _closing: bool) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn reset(&mut self) {}
    fn insert_above(&mut self, _s: String) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn resize(&mut self, _width: usize, _height: usize) {}
    fn clear_screen(&mut self) {}
    fn write_string(&mut self, _s: &str) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(0)
    }
    fn on_mouse(&mut self, _msg: MouseMsg) -> Cmd {
        None
    }

    fn set_optimizations(&mut self, _hard_tabs: bool, _backspace: bool, _map_nl: bool) {}

    fn set_color_profile(&mut self, _p: charming_colorprofile::Profile) {}
}
