//! Cleanroom Rust port of upstream Go example: `examples/textinputs/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple example demonstrating the use of multiple text input components
//! from the Bubbles component library.

use rusty_bubbles::cursor;
use rusty_bubbles::textinput;
use rusty_bubbletea::cursor::Cursor;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{batch, quit, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::{Color, Style};

fn focused_style() -> Style {
    Style::new().foreground("205")
}

fn blurred_style() -> Style {
    Style::new().foreground("240")
}

fn help_style() -> Style {
    blurred_style()
}

fn cursor_mode_help_style() -> Style {
    Style::new().foreground("244")
}

fn focused_button() -> String {
    focused_style().render("[ Submit ]")
}

fn blurred_button() -> String {
    format!("[ {} ]", blurred_style().render("Submit"))
}

struct Model {
    focus_index: usize,
    inputs: Vec<textinput::Model>,
    cursor_mode: cursor::Mode,
    quitting: bool,
}

impl Model {
    fn initial_model() -> Self {
        let mut m = Model {
            focus_index: 0,
            inputs: vec![textinput::new(), textinput::new(), textinput::new()],
            cursor_mode: cursor::Mode::Blink,
            quitting: false,
        };

        for i in 0..m.inputs.len() {
            m.inputs[i].char_limit = 32;

            let mut s = m.inputs[i].styles().clone();
            s.cursor.color = Color::parse("205");
            s.focused.prompt = focused_style();
            s.focused.text = focused_style();
            s.blurred.prompt = blurred_style();
            m.inputs[i].set_styles(s);

            match i {
                0 => {
                    m.inputs[i].placeholder = "Nickname".to_string();
                    m.inputs[i].focus();
                }
                1 => {
                    m.inputs[i].placeholder = "Email".to_string();
                    m.inputs[i].char_limit = 64;
                }
                2 => {
                    m.inputs[i].placeholder = "Password".to_string();
                    m.inputs[i].echo_mode = textinput::EchoMode::EchoPassword;
                    m.inputs[i].echo_character = '•';
                }
                _ => {}
            }
        }

        m
    }

    fn update_inputs(&mut self, msg: &dyn Msg) -> Cmd {
        let mut cmds: Vec<Cmd> = Vec::new();

        // Only text inputs with Focus() set will respond, so it's safe to
        // simply update all of them here without any further logic.
        for i in 0..self.inputs.len() {
            cmds.push(self.inputs[i].update(msg));
        }

        batch(cmds)
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        Some(Box::new(|| Some(textinput::blink())))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "esc" => {
                    self.quitting = true;
                    return quit();
                }

                // Change cursor mode
                "ctrl+r" => {
                    self.cursor_mode = match self.cursor_mode {
                        cursor::Mode::Blink => cursor::Mode::Static,
                        cursor::Mode::Static => cursor::Mode::Hide,
                        cursor::Mode::Hide => cursor::Mode::Blink,
                    };
                    for i in 0..self.inputs.len() {
                        let mut s = self.inputs[i].styles().clone();
                        s.cursor.blink = self.cursor_mode == cursor::Mode::Blink;
                        self.inputs[i].set_styles(s);
                    }
                    return None;
                }

                // Set focus to next input
                "tab" | "shift+tab" | "enter" | "up" | "down" => {
                    let s = k.0.to_string();

                    // Did the user press enter while the submit button was
                    // focused? If so, exit.
                    if s == "enter" && self.focus_index == self.inputs.len() {
                        return quit();
                    }

                    // Cycle indexes
                    if s == "up" || s == "shift+tab" {
                        if self.focus_index == 0 {
                            self.focus_index = self.inputs.len();
                        } else {
                            self.focus_index -= 1;
                        }
                    } else {
                        self.focus_index += 1;
                    }

                    if self.focus_index > self.inputs.len() {
                        self.focus_index = 0;
                    }

                    let mut cmds: Vec<Cmd> = Vec::new();
                    for i in 0..self.inputs.len() {
                        if i == self.focus_index {
                            // Set focused state
                            cmds.push(self.inputs[i].focus());
                            continue;
                        }
                        // Remove focused state
                        self.inputs[i].blur();
                    }

                    return batch(cmds);
                }
                _ => {}
            }
        }

        // Handle character input and blinking
        self.update_inputs(msg)
    }

    fn view(&self) -> View {
        let mut b = String::new();
        let mut c: Option<Cursor> = None;

        for (i, input) in self.inputs.iter().enumerate() {
            b += &self.inputs[i].view();
            if i < self.inputs.len() - 1 {
                b += "\n";
            }
            if self.cursor_mode != cursor::Mode::Hide && input.focused() {
                if let Some(mut cur) = input.cursor() {
                    cur.position.y += i;
                    c = Some(cur);
                }
            }
        }

        let button = if self.focus_index == self.inputs.len() {
            focused_button()
        } else {
            blurred_button()
        };
        b += &format!("\n\n{}\n\n", button);

        b += &help_style().render("cursor mode is ");
        b += &cursor_mode_help_style().render(self.cursor_mode.to_string());
        b += &help_style().render(" (ctrl+r to change style)");

        if self.quitting {
            b += "\n";
        }

        let mut v = View::new(&b);
        v.cursor = c;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::initial_model());
    p.run()?;
    Ok(())
}
