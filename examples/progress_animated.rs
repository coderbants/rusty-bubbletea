//! Cleanroom Rust port of upstream Go example: `examples/progress-animated/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple example that shows how to render an animated progress bar. In this
//! example we bump the progress by 25% every two seconds, animating our
//! progress bar to its new target state.
//!
//! It's also possible to render a progress bar in a more static fashion without
//! transitions. For details on that approach see the progress-static example.

use std::time::Duration;

use charming_bubbles::progress;
use charming_bubbletea::commands;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::screen::WindowSizeMsg;
use charming_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::{Color, Style};

const PADDING: usize = 2;
const MAX_WIDTH: usize = 80;

fn help_style() -> Style {
    Style::new().foreground_color(Color::parse("#626262"))
}

/// TickMsg is the message produced by the one-second timer.
#[derive(Debug)]
struct TickMsg;

fn tick_cmd() -> Cmd {
    commands::tick(Duration::from_secs(1), |_t| Some(Box::new(TickMsg)))
}

struct Model {
    progress: progress::Model,
}

impl Model {
    fn new() -> Self {
        Model {
            progress: progress::new(vec![progress::with_default_blend()]),
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
            if self.progress.percent() == 1.0 {
                return quit();
            }

            // Note that you can also use progress::Model::set_percent to set
            // the percentage value explicitly, too.
            let cmd = self.progress.incr_percent(0.25);
            return commands::batch(vec![tick_cmd(), cmd]);
        }

        // FrameMsg is sent when the progress bar wants to animate itself.
        if msg.as_any().downcast_ref::<progress::FrameMsg>().is_some() {
            return self.progress.update(msg);
        }

        None
    }

    fn view(&self) -> View {
        let pad = " ".repeat(PADDING);
        View::new(&format!(
            "\n{}{}\n\n{}{}",
            pad,
            self.progress.view(),
            pad,
            help_style().render("Press any key to quit")
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::new());
    p.run()?;
    Ok(())
}
