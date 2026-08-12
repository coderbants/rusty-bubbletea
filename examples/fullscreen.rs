//! Cleanroom Rust port of upstream Go example: `examples/fullscreen/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! Fullscreen example: a one-second tick counts a model down from 5 to 0,
//! then the program exits. The view renders in alternate screen mode.

use charming_bubbletea::{Cmd, KeyPressMsg, Model, Msg, Program, View};
use std::time::{Duration, SystemTime};

/// tickMsg is a message that represents a tick.
#[derive(Debug)]
struct TickMsg;

/// model is the countdown value.
#[derive(Clone, Copy)]
struct FullscreenModel(usize);

impl Model for FullscreenModel {
    fn init(&self) -> Cmd {
        tick()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "q" | "esc" | "ctrl+c" => return charming_bubbletea::quit(),
                _ => {}
            }
        }
        if msg.as_any().is::<TickMsg>() {
            self.0 = self.0.saturating_sub(1);
            if self.0 == 0 {
                return charming_bubbletea::quit();
            }
            return tick();
        }
        None
    }

    fn view(&self) -> View {
        let mut v = View::new(&format!(
            "\n\n     Hi. This program will exit in {} seconds...",
            self.0
        ));
        v.alt_screen = true;
        v
    }
}

fn tick() -> Cmd {
    charming_bubbletea::tick(Duration::from_secs(1), |_: SystemTime| {
        Some(Box::new(TickMsg))
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(FullscreenModel(5));
    program.run()?;
    Ok(())
}
