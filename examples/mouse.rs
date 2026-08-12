//! Cleanroom Rust port of upstream Go example: `examples/mouse/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple program that opens the alternate screen buffer and displays mouse
//! coordinates and events.

use charming_bubbletea::{
    print_ln, quit, Cmd, KeyPressMsg, Model, MouseButton, MouseClickMsg, MouseMode, Msg, Program,
    View,
};

/// Mouse string representation, mirroring the upstream `uv.Mouse.String()`
/// ("left", "right", etc.).
fn mouse_button_str(button: &MouseButton) -> &'static str {
    match button {
        MouseButton::MouseLeft => "left",
        MouseButton::MouseMiddle => "middle",
        MouseButton::MouseRight => "right",
        MouseButton::MouseWheelUp => "wheel up",
        MouseButton::MouseWheelDown => "wheel down",
        MouseButton::MouseWheelLeft => "wheel left",
        MouseButton::MouseWheelRight => "wheel right",
        MouseButton::MouseBackward => "backward",
        MouseButton::MouseForward => "forward",
        MouseButton::MouseButton10 => "button 10",
        MouseButton::MouseButton11 => "button 11",
        MouseButton::MouseNone => "none",
    }
}

struct MouseModel;

impl Model for MouseModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let s = k.0.to_string();
            if s == "ctrl+c" || s == "q" || s == "esc" {
                return quit();
            }
        }

        if let Some(m) = msg.as_any().downcast_ref::<MouseClickMsg>() {
            return print_ln(format_args!(
                "(X: {}, Y: {}) {}",
                m.0.x,
                m.0.y,
                mouse_button_str(&m.0.button)
            ));
        }
        None
    }

    fn view(&self) -> View {
        let mut v = View::new("Do mouse stuff. When you're done press q to quit.\n");
        v.mouse_mode = MouseMode::MouseModeAllMotion;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(MouseModel);
    program.run()?;
    Ok(())
}
