//! Cleanroom Rust port of upstream Go example: `examples/set-terminal-color/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! Choose a terminal-wide color (foreground, background or cursor) and set
//! it. All settings will be cleared on exit. Note that many terminals don't
//! support this.

use charming_bubbles::textinput;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::new_style;

/// ColorType is the kind of color to set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorType {
    Foreground,
    Background,
    Cursor,
}

impl ColorType {
    fn to_string(self) -> &'static str {
        match self {
            ColorType::Foreground => "Foreground",
            ColorType::Background => "Background",
            ColorType::Cursor => "Cursor",
        }
    }
}

/// State is the current UI state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Choose,
    Input,
}

struct Model {
    ti: textinput::Model,
    choice: Option<ColorType>,
    state: State,
    choice_index: usize,
    err: Option<String>,
    fg: Option<charming_bubbletea::color::Color>,
    bg: Option<charming_bubbletea::color::Color>,
    cc: Option<charming_bubbletea::color::Color>,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        Some(Box::new(|| Some(textinput::blink())))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "q" => return quit(),
                _ => {}
            }

            match self.state {
                State::Choose => {
                    self.ti.blur();
                    match k.0.to_string().as_str() {
                        "j" | "down" => {
                            self.choice_index += 1;
                            if self.choice_index > 2 {
                                self.choice_index = 0;
                            }
                        }
                        "k" | "up" => {
                            if self.choice_index == 0 {
                                self.choice_index = 2;
                            } else {
                                self.choice_index -= 1;
                            }
                        }
                        "enter" => {
                            self.state = State::Input;
                            let _ = self.ti.focus();
                            self.choice = match self.choice_index {
                                0 => Some(ColorType::Foreground),
                                1 => Some(ColorType::Background),
                                _ => Some(ColorType::Cursor),
                            };
                        }
                        _ => {}
                    }
                }
                State::Input => {
                    let _ = self.ti.focus();
                    match k.0.to_string().as_str() {
                        "esc" => {
                            self.choice = None;
                            self.choice_index = 0;
                            self.state = State::Choose;
                            self.err = None;
                            self.ti.blur();
                        }
                        "enter" => {
                            let val = self.ti.value();
                            match parse_hex_color(&val) {
                                Err(e) => self.err = Some(e),
                                Ok(col) => {
                                    self.err = None;
                                    let choice = self.choice.take();
                                    self.choice_index = 0;
                                    self.state = State::Choose;

                                    // Reset the text input.
                                    self.ti.reset();

                                    match choice {
                                        Some(ColorType::Foreground) => self.fg = Some(col),
                                        Some(ColorType::Background) => self.bg = Some(col),
                                        Some(ColorType::Cursor) => self.cc = Some(col),
                                        None => {}
                                    }
                                }
                            }

                            self.ti.blur();
                        }
                        _ => return self.ti.update(msg),
                    }
                }
            }
        }

        None
    }

    fn view(&self) -> View {
        let instructions = new_style()
            .width(40)
            .render("Choose a terminal-wide color to set. All settings will be cleared on exit.");

        let mut s = String::new();

        match self.state {
            State::Choose => {
                s += &(instructions + "\n\n");
                for (i, c) in [
                    ColorType::Foreground,
                    ColorType::Background,
                    ColorType::Cursor,
                ]
                .iter()
                .enumerate()
                {
                    if i == self.choice_index {
                        s += " > ";
                    } else {
                        s += "   ";
                    }
                    s += c.to_string();
                    s += "\n";
                }
            }
            State::Input => {
                s += "Enter a color in hex format:\n\n";
                s += &self.ti.view();
                s += "\n";
            }
        }

        if let Some(err) = &self.err {
            s += &format!("\nError: {}", err);
        }

        s += "\nPress q to quit";

        match self.state {
            State::Choose => s += ", j/k to move, and enter to select",
            State::Input => s += ", and enter to submit, esc to go back",
        }

        s += "\n";

        let mut v = View::new(&s);
        if self.ti.focused() {
            if let Some(mut c) = self.ti.cursor() {
                c.position.y += 2; // account for the prompt
                c.color = self.cc;
                v.cursor = Some(c);
            }
        }
        v.background_color = self.bg;
        v.foreground_color = self.fg;

        v
    }
}

/// ParseHexColor parses a hex color like `#ff00ff` or `ff00ff`, mirroring the
/// upstream `colorful.Hex`.
fn parse_hex_color(s: &str) -> Result<charming_bubbletea::color::Color, String> {
    let s = s.trim();
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return Err("invalid color".to_string());
    }
    let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| "invalid color".to_string())?;
    let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| "invalid color".to_string())?;
    let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| "invalid color".to_string())?;
    Ok(charming_x_ansi::color::RGBColor { r, g, b })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ti = textinput::new();
    ti.placeholder = "#ff00ff".to_string();
    ti.char_limit = 156;
    ti.set_width(20);
    ti.set_virtual_cursor(false);

    let p = Program::new(Model {
        ti,
        choice: None,
        state: State::Choose,
        choice_index: 0,
        err: None,
        fg: None,
        bg: None,
        cc: None,
    });
    p.run()?;
    Ok(())
}
