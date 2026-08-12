//! Cleanroom Rust port of upstream Go example: `examples/progress-bar/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A demo of the terminal's native progress bar, set on the declarative
//! `View`.

use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::screen::WindowSizeMsg;
use charming_bubbletea::{
    new_progress_bar, quit, Cmd, KeyPressMsg, Msg, Program, ProgressBarState, View,
};
use charming_lipgloss::Style;

fn body() -> Style {
    Style::new().padding(&[1, 2])
}

struct Model {
    value: usize,
    width: usize,
    state: ProgressBarState,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(w) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = w.width;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "q" | "ctrl+c" => return quit(),
                "up" | "k" => {
                    if self.value < 100 {
                        self.value += 10;
                    }
                }
                "down" | "j" => {
                    if self.value > 0 {
                        self.value -= 10;
                    }
                }
                "left" | "h" => {
                    let i = state_index(self.state);
                    if i > 0 {
                        self.state = state_from_index(i - 1);
                    }
                }
                "right" | "l" => {
                    let i = state_index(self.state);
                    if i < 4 {
                        self.state = state_from_index(i + 1);
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn view(&self) -> View {
        let b = body();
        let w = self.width.saturating_sub(b.get_horizontal_padding());
        let s = b.width(w).render(
            "This demo requires a terminal emulator that supports an indeterminate progress bar, such a Windows Terminal or Ghostty. In other terminals (including tmux in a supporting terminal) nothing will happen.\n\nPress up/down to change value, left/right to change state, q to quit.",
        );
        let mut v = View::new(&s);
        v.progress_bar = Some(new_progress_bar(self.state, self.value));
        v
    }
}

fn state_index(s: ProgressBarState) -> usize {
    match s {
        ProgressBarState::ProgressBarNone => 0,
        ProgressBarState::ProgressBarDefault => 1,
        ProgressBarState::ProgressBarError => 2,
        ProgressBarState::ProgressBarIndeterminate => 3,
        ProgressBarState::ProgressBarWarning => 4,
    }
}

fn state_from_index(i: usize) -> ProgressBarState {
    match i {
        0 => ProgressBarState::ProgressBarNone,
        1 => ProgressBarState::ProgressBarDefault,
        2 => ProgressBarState::ProgressBarError,
        3 => ProgressBarState::ProgressBarIndeterminate,
        _ => ProgressBarState::ProgressBarWarning,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model {
        value: 50,
        width: 0,
        state: ProgressBarState::ProgressBarIndeterminate,
    };
    let p = Program::new(m);
    p.run()?;
    Ok(())
}
