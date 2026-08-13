//! Cleanroom Rust port of upstream Go example: `examples/textinput/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple program demonstrating the text input component from the Bubbles
//! component library.

use rusty_bubbles::textinput;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};

struct Model {
    text_input: textinput::Model,
    quitting: bool,
}

impl Model {
    fn initial_model() -> Self {
        let mut ti = textinput::new();
        ti.placeholder = "Pikachu".to_string();
        ti.set_virtual_cursor(false);
        let _ = ti.focus();
        ti.char_limit = 156;
        ti.set_width(20);

        Model {
            text_input: ti,
            quitting: false,
        }
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        Some(Box::new(|| Some(textinput::blink())))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "enter" | "ctrl+c" | "esc" => {
                    self.quitting = true;
                    return quit();
                }
                _ => {}
            }
        }

        self.text_input.update(msg)
    }

    fn view(&self) -> View {
        let str = rusty_lipgloss::join::join_vertical(
            rusty_lipgloss::TOP,
            &[
                &self.header_view(),
                &self.text_input.view(),
                &self.footer_view(),
            ],
        );

        let mut v = View::new(&str);
        if self.quitting {
            v.set_content(&(str + "\n"));
        }
        // Real cursor (virtual cursor disabled): mirror upstream's
        // `v.Cursor = m.textInput.Cursor()` and the Y offset by the header
        // height (`c.Y += lipgloss.Height(m.headerView())`).
        if let Some(c) = self.text_input.cursor() {
            let mut c = c;
            c.position.y += rusty_lipgloss::size::height(&self.header_view());
            v.cursor = Some(c);
        }
        v
    }
}

impl Model {
    fn header_view(&self) -> String {
        "What’s your favorite Pokémon?\n".to_string()
    }

    fn footer_view(&self) -> String {
        "\n(esc to quit)".to_string()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::initial_model());
    p.run()?;
    Ok(())
}
