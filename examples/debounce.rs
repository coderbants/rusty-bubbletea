//! Cleanroom Rust port of upstream Go example: `examples/debounce/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! Debounce: every key press restarts a one-second timer carrying the
//! current tag; the program exits when a timer fires whose tag still
//! matches the latest press (i.e. after one second without input).

use rusty_bubbletea::{Cmd, KeyPressMsg, Model, Msg, Program, View};
use std::time::{Duration, SystemTime};

const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

/// exitMsg carries the tag value captured when the timer was armed.
#[derive(Debug, Clone)]
struct ExitMsg(#[allow(dead_code)] u32);

struct DebounceModel {
    tag: u32,
}

impl Model for DebounceModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(_k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            // Increment the tag on the model...
            self.tag += 1;
            // ...and include a copy of that tag value in the message.
            let tag = self.tag;
            return rusty_bubbletea::tick(DEBOUNCE_DURATION, move |_: SystemTime| {
                Some(Box::new(ExitMsg(tag)))
            });
        }
        if msg.as_any().is::<ExitMsg>() {
            let any = msg.as_any();
            let exit = any.downcast_ref::<ExitMsg>().unwrap();
            // If the tag in the message doesn't match the tag on the model
            // then this message was not the last one sent and another is on
            // the way. If that's the case, ignore it. Otherwise the debounce
            // timeout has passed and this message is a valid debounced one.
            if exit.0 == self.tag {
                return rusty_bubbletea::quit();
            }
        }
        None
    }

    fn view(&self) -> View {
        View::new(&format!(
            "Key presses: {}\nTo exit press any key, then wait for one second without pressing anything.",
            self.tag
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(DebounceModel { tag: 0 });
    program.run()?;
    Ok(())
}
