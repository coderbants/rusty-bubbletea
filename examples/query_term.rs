//! Cleanroom Rust port of upstream Go example: `examples/query-term/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! This example uses a textinput to send the terminal ANSI sequences to
//! query it for capabilities. Enter a quoted sequence like `"\x1b[>q"` and
//! press enter to write it to the terminal; responses are logged above the
//! program.

use std::io::Write as _;

use rusty_bubbles::textinput;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{print_f, quit, Cmd, KeyPressMsg, Msg, Program, View};

struct Model {
    input: textinput::Model,
    err: Option<String>,
}

impl Model {
    fn new() -> Self {
        let mut ti = textinput::new();
        let _ = ti.focus();
        ti.char_limit = 156;
        ti.set_width(20);
        ti.set_virtual_cursor(false);
        Model {
            input: ti,
            err: None,
        }
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        let mut cmds: Vec<Cmd> = Vec::new();

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            self.err = None;
            match k.0.to_string().as_str() {
                "ctrl+c" => return quit(),
                "enter" => {
                    // Write the sequence to the terminal.
                    let val = "\"".to_string() + &self.input.value() + "\"";

                    // Unescape the sequence.
                    let seq = match unescape(&val) {
                        Ok(s) => s,
                        Err(e) => {
                            self.err = Some(e);
                            return None;
                        }
                    };

                    if !seq.starts_with('\u{1b}') {
                        self.err = Some("sequence is not an ANSI escape sequence".to_string());
                        return None;
                    }

                    self.input.set_value("");

                    // Write the sequence to the terminal.
                    return Some(Box::new(move || {
                        let _ = std::io::stdout().write_all(seq.as_bytes());
                        None
                    }));
                }
                _ => {}
            }
        }

        if !msg.as_any().is::<KeyPressMsg>() {
            // Only log messages that are exported types, mirroring the
            // upstream `%T %+v` logging with Go's exact struct formatting.
            if let Some(s) = go_fmt(msg) {
                cmds.push(print_f(format_args!("Received message: {}", s)));
            }
        }

        let cmd = self.input.update(msg);
        cmds.push(cmd);

        rusty_bubbletea::batch(cmds)
    }

    fn view(&self) -> View {
        let mut s = self.input.view();
        if let Some(err) = &self.err {
            s += &format!("\n\nError: {}", err);
        }
        s += "\n\nPress ctrl+c to quit, enter to write the sequence to terminal";
        let mut v = View::new(&s);
        v.cursor = self.input.cursor();
        v
    }
}

/// Unescape removes the surrounding quotes and decodes escape sequences,
/// mirroring the upstream `strconv.Unquote` for the subset of escapes used
/// with terminal queries.
fn unescape(s: &str) -> Result<String, String> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 2 || chars[0] != '"' || chars[chars.len() - 1] != '"' {
        return Err("invalid syntax".to_string());
    }

    let mut out = String::new();
    let mut i = 1;
    while i < chars.len() - 1 {
        let c = chars[i];
        if c != '\\' {
            out.push(c);
            i += 1;
            continue;
        }
        if i + 1 >= chars.len() - 1 {
            return Err("invalid syntax".to_string());
        }
        match chars[i + 1] {
            '\\' => {
                out.push('\\');
                i += 2;
            }
            '"' => {
                out.push('"');
                i += 2;
            }
            'n' => {
                out.push('\n');
                i += 2;
            }
            'r' => {
                out.push('\r');
                i += 2;
            }
            't' => {
                out.push('\t');
                i += 2;
            }
            'x' => {
                if i + 3 >= chars.len() - 1 {
                    return Err("invalid syntax".to_string());
                }
                let hex: String = chars[i + 2..i + 4].iter().collect();
                let byte =
                    u8::from_str_radix(&hex, 16).map_err(|_| "invalid syntax".to_string())?;
                out.push(byte as char);
                i += 4;
            }
            _ => return Err("invalid syntax".to_string()),
        }
    }
    Ok(out)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::new());
    p.run()?;
    Ok(())
}

/// Formats a message the way Go's `%T %+v` would for the messages the
/// query-term example receives (upstream `log.Printf("Received message: %T %+v")`).
fn go_fmt(msg: &dyn rusty_bubbletea::Msg) -> Option<String> {
    use rusty_bubbletea::environ::EnvMsg;
    use rusty_bubbletea::profile::ColorProfileMsg;
    use rusty_bubbletea::screen::WindowSizeMsg;
    if let Some(m) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
        return Some(format!(
            "tea.WindowSizeMsg {{Width:{} Height:{}}}",
            m.width, m.height
        ));
    }
    if let Some(m) = msg.as_any().downcast_ref::<EnvMsg>() {
        let mut kv: Vec<String> = m.vars.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        kv.sort();
        return Some(format!("tea.EnvMsg [{}]", kv.join(" ")));
    }
    if let Some(m) = msg.as_any().downcast_ref::<ColorProfileMsg>() {
        let profile = match m.profile {
            rusty_bubbletea::profile::ColorProfile::TrueColor => "TrueColor",
            rusty_bubbletea::profile::ColorProfile::ANSI256 => "ANSI256",
            rusty_bubbletea::profile::ColorProfile::ANSI => "ANSI",
            rusty_bubbletea::profile::ColorProfile::Ascii => "Ascii",
        };
        return Some(format!("tea.ColorProfileMsg {}", profile));
    }
    None
}
