//! Cleanroom Rust port of upstream Go example: `examples/capability/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! Query the terminal for capabilities. Type a capability name like `TN`,
//! `RGB` or `cols` and press enter to request it. This will not work in all
//! terminals and multiplexers.

use rusty_bubbles::textinput;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::screen::WindowSizeMsg;
use rusty_bubbletea::termcap::CapabilityMsg;
use rusty_bubbletea::{print_f, quit, request_capability, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::new_style;

struct Model {
    input: textinput::Model,
    width: usize,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        Some(Box::new(|| Some(textinput::blink())))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            return None;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "esc" => return quit(),
                "enter" => {
                    let input = self.input.value();
                    self.input.reset();
                    return request_capability(&input);
                }
                _ => {}
            }
        }

        if let Some(cap) = msg.as_any().downcast_ref::<CapabilityMsg>() {
            return print_f(format_args!("Got capability: {}", cap));
        }

        self.input.update(msg)
    }

    fn view(&self) -> View {
        let w = self.width.min(60);

        let instructions = new_style()
            .width(w)
            .render("Query for terminal capabilities. You can enter things like 'TN', 'RGB', 'cols', and so on. This will not work in all terminals and multiplexers.");

        View::new(&format!(
            "\n{}\n\n{}\n\nPress enter to request capability, or ctrl+c to quit.",
            instructions,
            self.input.view()
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = textinput::new();
    input.placeholder = "Enter capability name to request".to_string();
    let _ = input.focus();

    let p = Program::new(Model { input, width: 0 });
    p.run()?;
    Ok(())
}
