//! Cleanroom Rust port of upstream Go example: `examples/altscreen-toggle/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! Demonstrates switching between the inline and alternate screen modes.

use rusty_bubbletea::{quit, suspend, Cmd, KeyPressMsg, Msg, Program, ResumeMsg, View};
use rusty_lipgloss::{new_style, Color, Style};

/// Keyword style, mirroring the upstream `keywordStyle`.
fn keyword_style() -> Style {
    new_style()
        .foreground_color(Color::parse("204"))
        .background_color(Color::parse("235"))
}

/// Help style, mirroring the upstream `helpStyle`.
fn help_style() -> Style {
    new_style().foreground_color(Color::parse("241"))
}

struct Model {
    altscreen: bool,
    quitting: bool,
    suspending: bool,
}

impl rusty_bubbletea::model::Model for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().is::<ResumeMsg>() {
            self.suspending = false;
            return None;
        }
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let key_str = k.0.to_string();
            match key_str.as_str() {
                "q" | "ctrl+c" | "esc" => {
                    self.quitting = true;
                    return quit();
                }
                "ctrl+z" => {
                    self.suspending = true;
                    return suspend();
                }
                "space" => {
                    self.altscreen = !self.altscreen;
                    return None;
                }
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> View {
        if self.suspending {
            let mut v = View::new("");
            v.alt_screen = self.altscreen;
            return v;
        }

        if self.quitting {
            let mut v = View::new("Bye!\n");
            v.alt_screen = self.altscreen;
            return v;
        }

        let altscreen_mode = " altscreen mode ";
        let inline_mode = " inline mode ";
        let mode = if self.altscreen {
            altscreen_mode
        } else {
            inline_mode
        };

        let content = format!(
            "\n\n  You're in {}\n\n\n{}",
            keyword_style().render(mode),
            help_style().render("  space: switch modes • ctrl-z: suspend • q: exit\n")
        );

        let mut v = View::new(&content);
        v.alt_screen = self.altscreen;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(Model {
        altscreen: false,
        quitting: false,
        suspending: false,
    });
    program.run()?;
    Ok(())
}
