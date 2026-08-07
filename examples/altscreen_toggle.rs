//! Cleanroom Rust port of upstream Go example: `examples/altscreen-toggle/main.go`
//! Upstream Target Tag / Version: `v1.3.4`

use charming_bubbletea::*;
use charming_lipgloss::Style;

struct AltScreenModel {
    altscreen: bool,
    quitting: bool,
}

impl Model for AltScreenModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(key) = msg.as_ref().as_any().downcast_ref::<KeyMsg>() {
            match key.to_string_rep().as_str() {
                "q" | "ctrl+c" | "esc" => {
                    self.quitting = true;
                    return quit();
                }
                " " => {
                    let cmd = if self.altscreen {
                        exit_alt_screen()
                    } else {
                        enter_alt_screen()
                    };
                    self.altscreen = !self.altscreen;
                    return cmd;
                }
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> String {
        if self.quitting {
            return "Bye!\n".to_string();
        }

        let keyword_style = Style::new().foreground("204").background("235");
        let help_style = Style::new().foreground("241");

        let mode = if self.altscreen {
            " altscreen mode "
        } else {
            " inline mode "
        };

        format!(
            "\n\n  You're in {}\n\n\n{}\n",
            keyword_style.render(mode),
            help_style.render("  space: switch modes • q: exit")
        )
    }
}

fn main() {
    let p = Program::new(AltScreenModel {
        altscreen: false,
        quitting: false,
    });
    if let Err(err) = p.run() {
        eprintln!("Error running program: {}", err);
        std::process::exit(1);
    }
}
