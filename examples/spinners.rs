//! Cleanroom Rust port of upstream Go example: `examples/spinners/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A program that cycles through all the available spinner styles.

use rusty_bubbles::spinner;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::Style;

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

fn text_style() -> Style {
    Style::new().foreground("252")
}

fn spinner_style() -> Style {
    Style::new().foreground("69")
}

fn help_style() -> Style {
    Style::new().foreground("241")
}

struct Model {
    index: usize,
    spinner: spinner::Model,
}

impl Model {
    fn reset_spinner(&mut self) {
        self.spinner = spinner::new(vec![]);
        self.spinner.style = spinner_style();
        self.spinner.spinner = spinners()[self.index].clone();
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        let tm = self.spinner.tick_msg();
        Some(Box::new(move || Some(Box::new(tm))))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "q" | "esc" => return quit(),
                "h" | "left" => {
                    if self.index == 0 {
                        self.index = spinners().len() - 1;
                    } else {
                        self.index -= 1;
                    }
                    self.reset_spinner();
                    let tm = self.spinner.tick_msg();
                    return Some(Box::new(move || Some(Box::new(tm))));
                }
                "l" | "right" => {
                    self.index += 1;
                    if self.index >= spinners().len() {
                        self.index = 0;
                    }
                    self.reset_spinner();
                    let tm = self.spinner.tick_msg();
                    return Some(Box::new(move || Some(Box::new(tm))));
                }
                _ => return None,
            }
        }

        if msg.as_any().downcast_ref::<spinner::TickMsg>().is_some() {
            return self.spinner.update(msg);
        }

        None
    }

    fn view(&self) -> View {
        let gap = if self.index == 1 { "" } else { " " };

        let mut s = String::new();
        s += &format!(
            "\n {}{}{}\n\n",
            self.spinner.view(),
            gap,
            text_style().render("Spinning..."),
        );
        s += &help_style().render("h/l, ←/→: change spinner • q: exit\n");
        View::new(&s)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut m = Model {
        index: 0,
        spinner: spinner::new(vec![]),
    };
    m.reset_spinner();

    let p = Program::new(m);
    p.run()?;
    Ok(())
}
