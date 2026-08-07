//! Cleanroom Rust port of upstream Go source file: `renderer.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Renderer
//!
//! Renderer interface trait for Bubble Tea terminal output renderers.
//! </public-docs>

/// <upstream-comment>
/// Renderer is the interface for Bubble Tea renderers.
/// </upstream-comment>
pub trait Renderer: Send + Sync {
    /// Start the renderer.
    fn start(&mut self);

    /// Stop the renderer, but render the final frame in the buffer, if any.
    fn stop(&mut self);

    /// Stop the renderer without doing any final rendering.
    fn kill(&mut self);

    /// Write a frame to the renderer.
    fn write(&mut self, s: String);

    /// Request a full re-render.
    fn repaint(&mut self);

    /// Clears the terminal.
    fn clear_screen(&mut self);

    /// Whether or not the alternate screen buffer is enabled.
    fn alt_screen(&self) -> bool;

    /// Enable the alternate screen buffer.
    fn enter_alt_screen(&mut self);

    /// Disable the alternate screen buffer.
    fn exit_alt_screen(&mut self);

    /// Show the cursor.
    fn show_cursor(&mut self);

    /// Hide the cursor.
    fn hide_cursor(&mut self);

    /// Enable mouse cell motion.
    fn enable_mouse_cell_motion(&mut self);

    /// Disable mouse cell motion.
    fn disable_mouse_cell_motion(&mut self);

    /// Enable mouse all motion.
    fn enable_mouse_all_motion(&mut self);

    /// Disable mouse all motion.
    fn disable_mouse_all_motion(&mut self);

    /// Enable mouse SGR mode.
    fn enable_mouse_sgr_mode(&mut self);

    /// Disable mouse SGR mode.
    fn disable_mouse_sgr_mode(&mut self);

    /// Enable bracketed paste.
    fn enable_bracketed_paste(&mut self);

    /// Disable bracketed paste.
    fn disable_bracketed_paste(&mut self);

    /// Reports whether bracketed paste mode is active.
    fn bracketed_paste_active(&self) -> bool;

    /// Set window title.
    fn set_window_title(&mut self, title: &str);

    /// Reports whether focus reporting is enabled.
    fn report_focus(&self) -> bool;

    /// Enable report focus.
    fn enable_report_focus(&mut self);

    /// Disable report focus.
    fn disable_report_focus(&mut self);
}

/// <upstream-comment>
/// RepaintMsg forces a full repaint.
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepaintMsg;
