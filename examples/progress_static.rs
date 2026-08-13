//! Cleanroom Rust port of upstream Go example: `examples/progress-static/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple example that shows how to render a progress bar in a "pure"
//! fashion. In this example we bump the progress by 25% every second,
//! maintaining the progress state on our top level model using the progress
//! bar model's `view_as` method only for rendering.
//!
//! `view_as(percent)` takes a float between 0 and 1, and renders the progress
//! bar accordingly. When using the progress bar in this "pure" fashion
//! there's no need to call an update method.

use std::time::Duration;

use rusty_bubbles::progress;
use rusty_bubbletea::commands;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::screen::WindowSizeMsg;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::{Color, Style};

const PADDING: usize = 2;
const MAX_WIDTH: usize = 80;

/// TickMsg is the message produced by the one-second timer.
#[derive(Debug)]
struct TickMsg;

fn tick_cmd() -> Cmd {
    commands::tick(Duration::from_secs(1), |_t| Some(Box::new(TickMsg)))
}

struct Model {
    percent: f64,
    progress: progress::Model,
    help_style: Style,
}

impl Model {
    fn new() -> Self {
        let prog = progress::new(vec![
            progress::with_scaled(true),
            progress::with_colors(&[Color::parse("#FF7CCB"), Color::parse("#FDFF8C")]),
        ]);
        Model {
            percent: 0.0,
            progress: prog,
            help_style: Style::new().foreground_color(Color::parse("#626262")),
        }
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        tick_cmd()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().downcast_ref::<KeyPressMsg>().is_some() {
            return quit();
        }

        if let Some(w) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            let width = w.width.saturating_sub(PADDING * 2 + 4);
            self.progress.set_width(width);
            if self.progress.width() > MAX_WIDTH {
                self.progress.set_width(MAX_WIDTH);
            }
            return None;
        }

        if msg.as_any().is::<TickMsg>() {
            self.percent += 0.25;
            if self.percent > 1.0 {
                self.percent = 1.0;
                return quit();
            }
            return tick_cmd();
        }

        None
    }

    fn view(&self) -> View {
        let pad = " ".repeat(PADDING);
        View::new(&format!(
            "\n{}{}\n\n{}{}",
            pad,
            self.progress.view_as(self.percent),
            pad,
            self.help_style.render("Press any key to quit")
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::new());
    p.run()?;
    Ok(())
}
