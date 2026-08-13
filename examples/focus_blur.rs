//! Cleanroom Rust port of upstream Go example: `examples/focus-blur/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple program that handles losing and acquiring focus, with focus
//! reporting toggled by the `t` key.

use rusty_bubbletea::focus::{BlurMsg, FocusMsg};
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};

/// A model tracking the focus state and whether focus reporting is enabled.
struct Model {
    focused: bool,
    reporting: bool,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().is::<FocusMsg>() {
            self.focused = true;
        } else if msg.as_any().is::<BlurMsg>() {
            self.focused = false;
        } else if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "t" => self.reporting = !self.reporting,
                "ctrl+c" | "q" => return quit(),
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> View {
        let mut s = String::from("Hi. Focus report is currently ");
        if self.reporting {
            s += "enabled";
        } else {
            s += "disabled";
        }
        s += ".\n\n";

        if self.reporting {
            if self.focused {
                s += "This program is currently focused!";
            } else {
                s += "This program is currently blurred!";
            }
        }

        let mut v = View::new(&format!(
            "{}\n\nTo quit sooner press ctrl-c, or t to toggle focus reporting...\n",
            s
        ));
        v.report_focus = self.reporting;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model {
        focused: true,
        reporting: true,
    });
    p.run()?;
    Ok(())
}
