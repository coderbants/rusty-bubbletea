//! Cleanroom Rust port of upstream Go example: `examples/vanish/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A program that quits and vanishes without a trace when any key is pressed.

use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};

/// The model is a simple boolean that records whether a key has been pressed,
/// mirroring `type model bool`.
struct Model(bool);

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().downcast_ref::<KeyPressMsg>().is_some() {
            self.0 = true;
            return quit();
        }
        None
    }

    fn view(&self) -> View {
        if self.0 {
            return View::new("");
        }
        View::new(
            "Press any key to quit.\n(When this program quits, it will vanish without a trace.)",
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model(false));
    p.run()?;
    Ok(())
}
