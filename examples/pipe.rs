//! Cleanroom Rust port of upstream Go example: `examples/pipe/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! An example illustrating how to pipe in data to a Bubble Tea application.
//! More so, this serves as proof that Bubble Tea will automatically listen for
//! keystrokes when input is not a TTY, such as when data is piped or
//! redirected in.
//!
//! As in the upstream example, all of stdin is read before the program
//! starts; the program itself then renders the piped content in a text input
//! and waits for input (which, when piped, never arrives — quit with the
//! terminal's interrupt instead).

use std::io::{IsTerminal, Read};

use charming_bubbles::textinput;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::Color;

/// A model wrapping a text input pre-filled with the piped content.
struct Model {
    user_input: textinput::Model,
}

/// Builds the model, mirroring `newModel` in the upstream example: no prompt,
/// a cursor colored "63", a width of 48, the piped content as the value, and
/// the cursor at the end of the text.
fn new_model(initial_value: &str) -> Model {
    let mut i = textinput::new();
    i.prompt = String::new();

    let mut s = i.styles().clone();
    s.cursor.color = Color::parse("63");
    i.set_styles(s);

    i.set_width(48);
    i.set_value(initial_value);
    i.cursor_end();
    i.focus();

    Model { user_input: i }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        // Start the cursor blinking, mirroring `textinput.Blink`.
        Some(Box::new(|| Some(textinput::blink())))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "esc" | "enter" => return quit(),
                _ => {}
            }
        }

        self.user_input.update(msg)
    }

    fn view(&self) -> View {
        View::new(&format!(
            "\nYou piped in: {}\n\nPress ^C to exit",
            self.user_input.view()
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // When running interactively (no pipe), there's nothing to show.
    if std::io::stdin().is_terminal() {
        println!("Try piping in some text.");
        std::process::exit(1);
    }

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let p = Program::new(new_model(input.trim()));
    p.run()?;
    Ok(())
}
