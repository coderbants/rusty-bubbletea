//! Cleanroom Rust port of upstream Go example: `examples/simple/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple program that counts down from 5 and then exits.

use std::time::Duration;

use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::{quit, suspend, Cmd, KeyPressMsg, Msg, Program, View};

/// A model can be more or less any type of data. It holds all the data for a
/// program, so often it's a struct. For this simple example, however, all
/// we'll need is a simple integer.
struct CountdownModel {
    countdown: usize,
}

/// Messages are events that we respond to in our Update function. This
/// particular one indicates that the timer has ticked.
#[derive(Debug)]
struct TickMsg;

impl CountdownModel {
    fn new(n: usize) -> Self {
        CountdownModel { countdown: n }
    }

    fn tick() -> Cmd {
        Some(Box::new(|| {
            std::thread::sleep(Duration::from_secs(1));
            Some(Box::new(TickMsg))
        }))
    }
}

impl ModelTrait for CountdownModel {
    /// Init optionally returns an initial command we should run. In this case
    /// we want to start the timer.
    fn init(&self) -> Cmd {
        Self::tick()
    }

    /// Update is called when messages are received. The idea is that you
    /// inspect the message and send back an updated model accordingly. You can
    /// also return a command, which is a function that performs I/O and
    /// returns a message.
    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "q" => return quit(),
                "ctrl+z" => return suspend(),
                _ => {}
            }
        }

        if msg.as_any().is::<TickMsg>() {
            self.countdown = self.countdown.saturating_sub(1);
            if self.countdown == 0 {
                return quit();
            }
            return Self::tick();
        }

        None
    }

    /// View returns a string based on data in the model. That string which
    /// will be rendered to the terminal.
    fn view(&self) -> View {
        View::new(&format!(
            "Hi. This program will exit in {} seconds.\n\nTo quit sooner press ctrl-c, or press ctrl-z to suspend...\n",
            self.countdown
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(CountdownModel::new(5));
    p.run()?;
    Ok(())
}
