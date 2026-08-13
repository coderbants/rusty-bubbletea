//! Cleanroom Rust port of upstream Go example: `examples/composable-views/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A program that composes two independent Bubble Tea components (a timer and
//! a spinner) side by side, with tab to switch focus and "n" to reset the
//! focused component.
//!
//! This example assumes an existing understanding of commands and messages. If
//! you haven't already read our tutorials on the basics of Bubble Tea and
//! working with commands, we recommend reading those first.

use std::time::Duration;

use rusty_bubbles::spinner;
use rusty_bubbles::timer;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{batch, quit, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::border;
use rusty_lipgloss::join::join_horizontal;
use rusty_lipgloss::{Style, TOP};

const DEFAULT_TIME: Duration = Duration::from_secs(60);

/// SessionState is used to track which model is focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionState {
    TimerView,
    SpinnerView,
}

/// The available spinners, mirroring the upstream list.
fn spinners() -> Vec<spinner::Spinner> {
    vec![
        spinner::line(),
        spinner::dot(),
        spinner::mini_dot(),
        spinner::jump(),
        spinner::pulse(),
        spinner::points(),
        spinner::globe(),
        spinner::moon(),
        spinner::monkey(),
    ]
}

fn model_style() -> Style {
    Style::new()
        .width(15)
        .height(5)
        .align(&[rusty_lipgloss::CENTER, rusty_lipgloss::CENTER])
        .border(border::hidden_border(), &[true, true, true, true])
}

fn focused_model_style() -> Style {
    Style::new()
        .width(15)
        .height(5)
        .align(&[rusty_lipgloss::CENTER, rusty_lipgloss::CENTER])
        .border(border::normal_border(), &[true, true, true, true])
        .border_foreground(&["69"])
}

fn spinner_style() -> Style {
    Style::new().foreground("69")
}

fn help_style() -> Style {
    Style::new().foreground("241")
}

struct MainModel {
    state: SessionState,
    timer: timer::Model,
    spinner: spinner::Model,
    index: usize,
}

impl MainModel {
    fn new_model(timeout: Duration) -> Self {
        MainModel {
            state: SessionState::TimerView,
            timer: timer::new(timeout, vec![]),
            spinner: spinner::new(vec![]),
            index: 0,
        }
    }

    fn current_focused_model(&self) -> &'static str {
        if self.state == SessionState::TimerView {
            return "timer";
        }
        "spinner"
    }

    fn next(&mut self) {
        if self.index == spinners().len() - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }

    fn reset_spinner(&mut self) {
        self.spinner = spinner::new(vec![]);
        self.spinner.style = spinner_style();
        self.spinner.spinner = spinners()[self.index].clone();
    }
}

impl ModelTrait for MainModel {
    fn init(&self) -> Cmd {
        // Start the timer and spinner on program start.
        let mut t = self.timer.clone();
        let tm = self.spinner.tick_msg();
        batch(vec![t.init(), Some(Box::new(move || Some(Box::new(tm))))])
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        let mut cmds: Vec<Cmd> = Vec::new();

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "q" => return quit(),
                "tab" => {
                    self.state = match self.state {
                        SessionState::TimerView => SessionState::SpinnerView,
                        SessionState::SpinnerView => SessionState::TimerView,
                    };
                }
                "n" => match self.state {
                    SessionState::TimerView => {
                        self.timer = timer::new(DEFAULT_TIME, vec![]);
                        let mut t = self.timer.clone();
                        cmds.push(t.init());
                    }
                    SessionState::SpinnerView => {
                        self.next();
                        self.reset_spinner();
                        let tm = self.spinner.tick_msg();
                        cmds.push(Some(Box::new(move || Some(Box::new(tm)))));
                    }
                },
                _ => {}
            }

            // Update whichever model is focused.
            match self.state {
                SessionState::SpinnerView => {
                    cmds.push(self.spinner.update(msg));
                }
                SessionState::TimerView => {
                    cmds.push(self.timer.update(msg));
                }
            }
        }

        if msg.as_any().downcast_ref::<spinner::TickMsg>().is_some() {
            cmds.push(self.spinner.update(msg));
        }

        if msg.as_any().downcast_ref::<timer::TickMsg>().is_some() {
            cmds.push(self.timer.update(msg));
        }

        batch(cmds)
    }

    fn view(&self) -> View {
        let model = self.current_focused_model();
        let timer_view = format!("{:>4}", self.timer.view());
        let spinner_view = self.spinner.view();

        let mut s = String::new();
        if self.state == SessionState::TimerView {
            s += &join_horizontal(
                TOP,
                &[
                    &focused_model_style().render(&timer_view),
                    &model_style().render(&spinner_view),
                ],
            );
        } else {
            s += &join_horizontal(
                TOP,
                &[
                    &model_style().render(&timer_view),
                    &focused_model_style().render(&spinner_view),
                ],
            );
        }
        s += &help_style().render(&format!("\ntab: focus next • n: new {} • q: exit\n", model));
        View::new(&s)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(MainModel::new_model(DEFAULT_TIME));
    p.run()?;
    Ok(())
}
