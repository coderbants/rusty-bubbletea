//! Cleanroom Rust port of upstream Go example: `examples/set-window-title/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple example illustrating how to set a window title.

use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};

/// The window title we'd like to set.
const WINDOW_TITLE: &str = "Hello, Bubble Tea";

/// An empty model: this program has no state.
struct Model;

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().is::<KeyPressMsg>() {
            return quit();
        }
        None
    }

    fn view(&self) -> View {
        let wrap = rusty_lipgloss::new_style().width(78);
        let mut v = View::new(&format!(
            "{}\n\nPress any key to quit.",
            wrap.render(&format!(
                "The window title has been set to '{}'. It will be cleared on exit.",
                WINDOW_TITLE
            ))
        ));
        v.window_title = WINDOW_TITLE.to_string();
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model);
    p.run()?;
    Ok(())
}
