//! Cleanroom Rust port of upstream Go example: `examples/stopwatch/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A stopwatch with start/stop/reset controls and a help view.

use std::time::Duration;

use rusty_bubbles::help;
use rusty_bubbles::key::{self, Binding};
use rusty_bubbles::stopwatch;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};

struct Model {
    stopwatch: stopwatch::Model,
    keymap: Keymap,
    help: help::Model,
    quitting: bool,
}

struct Keymap {
    start: Binding,
    stop: Binding,
    reset: Binding,
    quit: Binding,
}

impl Model {
    fn new() -> Self {
        let mut keymap = Keymap {
            start: key::new_binding(vec![key::with_keys(&["s"]), key::with_help("s", "start")]),
            stop: key::new_binding(vec![key::with_keys(&["s"]), key::with_help("s", "stop")]),
            reset: key::new_binding(vec![key::with_keys(&["r"]), key::with_help("r", "reset")]),
            quit: key::new_binding(vec![
                key::with_keys(&["ctrl+c", "q"]),
                key::with_help("q", "quit"),
            ]),
        };
        keymap.start.set_enabled(false);
        Model {
            stopwatch: stopwatch::new(vec![stopwatch::with_interval(Duration::from_millis(1))]),
            keymap,
            help: help::new(),
            quitting: false,
        }
    }

    fn help_view(&self) -> String {
        let mut s = String::from("\n");
        s += &self.help.short_help_view(&[
            self.keymap.start.clone(),
            self.keymap.stop.clone(),
            self.keymap.reset.clone(),
            self.keymap.quit.clone(),
        ]);
        s
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        let mut s = self.stopwatch.clone();
        s.init()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let s = k.0.to_string();
            if key::matches(&s, std::slice::from_ref(&self.keymap.quit)) {
                self.quitting = true;
                return quit();
            }
            if key::matches(&s, std::slice::from_ref(&self.keymap.reset)) {
                return self.stopwatch.reset();
            }
            if key::matches(&s, &[self.keymap.start.clone(), self.keymap.stop.clone()]) {
                self.keymap.stop.set_enabled(!self.stopwatch.running());
                self.keymap.start.set_enabled(self.stopwatch.running());
                return self.stopwatch.toggle();
            }
        }

        self.stopwatch.update(msg)
    }

    fn view(&self) -> View {
        // Note: you could further customize the time output by getting the
        // duration from m.stopwatch.elapsed(), which returns a Duration, and
        // skip m.stopwatch.view() altogether.
        let mut s = self.stopwatch.view() + "\n";
        if !self.quitting {
            s = "Elapsed: ".to_string() + &s;
            s += &self.help_view();
        }
        View::new(&s)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::new());
    p.run()?;
    Ok(())
}
