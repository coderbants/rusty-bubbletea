//! Cleanroom Rust port of upstream Go example: `examples/timer/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A program that runs a countdown timer, with start/stop/reset controls.

use std::time::Duration;

use charming_bubbles::help;
use charming_bubbles::key::{self, Binding};
use charming_bubbles::timer;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};

const TIMEOUT: Duration = Duration::from_secs(5);

struct Model {
    timer: timer::Model,
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
                key::with_keys(&["q", "ctrl+c"]),
                key::with_help("q", "quit"),
            ]),
        };
        keymap.start.set_enabled(false);
        Model {
            timer: timer::new(
                TIMEOUT,
                vec![timer::with_interval(Duration::from_millis(1))],
            ),
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
        let mut t = self.timer.clone();
        t.init()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(_m) = msg.as_any().downcast_ref::<timer::TickMsg>() {
            return self.timer.update(msg);
        }

        if msg.as_any().downcast_ref::<timer::StartStopMsg>().is_some() {
            let cmd = self.timer.update(msg);
            self.keymap.stop.set_enabled(self.timer.running());
            self.keymap.start.set_enabled(!self.timer.running());
            return cmd;
        }

        if msg.as_any().downcast_ref::<timer::TimeoutMsg>().is_some() {
            self.quitting = true;
            return quit();
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let kk = &k.0;
            if key::matches(kk, std::slice::from_ref(&self.keymap.quit)) {
                self.quitting = true;
                return quit();
            }
            if key::matches(kk, std::slice::from_ref(&self.keymap.reset)) {
                self.timer.timeout = TIMEOUT;
            }
            if key::matches(kk, &[self.keymap.start.clone(), self.keymap.stop.clone()]) {
                return self.timer.toggle();
            }
        }

        None
    }

    fn view(&self) -> View {
        // For a more detailed timer view you could read m.timer.Timeout to
        // get the remaining time as a Duration and skip calling
        // m.timer.view() entirely.
        let mut s = self.timer.view();

        if self.timer.timedout() {
            s = "All done!".to_string();
        }
        s += "\n";
        if !self.quitting {
            s = format!("Exiting in {}", s);
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
