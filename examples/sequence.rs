//! Cleanroom Rust port of upstream Go example: `examples/sequence/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! Demonstrates `tea.Sequence` vs `tea.Batch`: commands in a sequence run
//! one at a time in order; commands in a batch run concurrently. A tree of
//! both prints labelled lines with sleeps in between, then quits.

use rusty_bubbletea::{
    batch, print_ln, quit, sequence, Cmd, KeyPressMsg, Model, Msg, Program, View,
};
use std::time::Duration;

struct SequenceModel;

impl Model for SequenceModel {
    fn init(&self) -> Cmd {
        // A sequence runs its commands one at a time, in order. Contrast
        // this with a batch, which runs commands concurrently.
        sequence(vec![
            batch(vec![
                sequence(vec![
                    sleep_println("1-1-1", 1000),
                    sleep_println("1-1-2", 1000),
                ]),
                batch(vec![
                    sleep_println("1-2-1", 1500),
                    sleep_println("1-2-2", 1250),
                ]),
            ]),
            print_ln(format_args!("2")),
            sequence(vec![
                batch(vec![
                    sleep_println("3-1-1", 500),
                    sleep_println("3-1-2", 1000),
                ]),
                sequence(vec![
                    sleep_println("3-2-1", 750),
                    sleep_println("3-2-2", 500),
                ]),
            ]),
            quit(),
        ])
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().downcast_ref::<KeyPressMsg>().is_some() {
            return quit();
        }
        None
    }

    fn view(&self) -> View {
        View::new("")
    }
}

/// sleep_println prints a string after stopping for a certain period of time.
fn sleep_println(s: &str, milliseconds: u64) -> Cmd {
    let s = s.to_string();
    Some(Box::new(move || {
        std::thread::sleep(Duration::from_millis(milliseconds));
        // Defer to the println command's message.
        print_ln(format_args!("{s}"))?()
    }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(SequenceModel);
    program.run()?;
    Ok(())
}
