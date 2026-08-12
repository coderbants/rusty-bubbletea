//! Cleanroom Rust port of upstream Go example: `examples/spinner/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple program demonstrating the spinner component from the Bubbles
//! component library.

use charming_bubbles::spinner;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::Style;

/// An error message. It's part of the upstream example's message handling,
/// though in practice nothing produces this message.
#[derive(Debug)]
struct ErrMsg;

struct Model {
    spinner: spinner::Model,
    quitting: bool,
    err: Option<String>,
}

impl Model {
    fn initial_model() -> Self {
        let mut s = spinner::new(vec![]);
        s.spinner = spinner::dot();
        s.style = Style::new().foreground("205");
        Model {
            spinner: s,
            quitting: false,
            err: None,
        }
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
                "q" | "esc" | "ctrl+c" => {
                    self.quitting = true;
                    return quit();
                }
                _ => return None,
            }
        }

        if msg.as_any().is::<ErrMsg>() {
            self.err = Some("(error)".to_string());
            return None;
        }

        self.spinner.update(msg)
    }

    fn view(&self) -> View {
        if let Some(err) = &self.err {
            return View::new(err);
        }
        let str = format!(
            "\n\n   {} Loading forever...press q to quit\n\n",
            self.spinner.view()
        );
        if self.quitting {
            return View::new(&(str + "\n"));
        }
        View::new(&str)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::initial_model());
    p.run()?;
    Ok(())
}
