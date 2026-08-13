//! Cleanroom Rust port of upstream Go example: `examples/help/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A program demonstrating the help component, with a custom keymap and
//! short/full help views.

use rusty_bubbles::help::{self, KeyMap as KeyMapTrait};
use rusty_bubbles::key::{self, Binding};
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::screen::WindowSizeMsg;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::{Color, Style};

/// KeyMap defines a set of keybindings. To work for help it must satisfy
/// the `help::KeyMap` trait.
#[derive(Clone)]
struct KeyMap {
    up: Binding,
    down: Binding,
    left: Binding,
    right: Binding,
    help: Binding,
    quit: Binding,
}

/// ShortHelp returns keybindings to be shown in the mini help view. It's part
/// of the `help::KeyMap` trait.
impl KeyMapTrait for KeyMap {
    fn short_help(&self) -> Vec<Binding> {
        vec![self.help.clone(), self.quit.clone()]
    }

    /// FullHelp returns keybindings for the expanded help view. It's part of
    /// the `help::KeyMap` trait.
    fn full_help(&self) -> Vec<Vec<Binding>> {
        vec![
            vec![
                self.up.clone(),
                self.down.clone(),
                self.left.clone(),
                self.right.clone(),
            ], // first column
            vec![self.help.clone(), self.quit.clone()], // second column
        ]
    }
}

fn keys() -> KeyMap {
    KeyMap {
        up: key::new_binding(vec![
            key::with_keys(&["up", "k"]),
            key::with_help("↑/k", "move up"),
        ]),
        down: key::new_binding(vec![
            key::with_keys(&["down", "j"]),
            key::with_help("↓/j", "move down"),
        ]),
        left: key::new_binding(vec![
            key::with_keys(&["left", "h"]),
            key::with_help("←/h", "move left"),
        ]),
        right: key::new_binding(vec![
            key::with_keys(&["right", "l"]),
            key::with_help("→/l", "move right"),
        ]),
        help: key::new_binding(vec![
            key::with_keys(&["?"]),
            key::with_help("?", "toggle help"),
        ]),
        quit: key::new_binding(vec![
            key::with_keys(&["q", "esc", "ctrl+c"]),
            key::with_help("q", "quit"),
        ]),
    }
}

struct Model {
    keys: KeyMap,
    help: help::Model,
    input_style: Style,
    last_key: String,
    quitting: bool,
}

impl Model {
    fn new_model() -> Self {
        Model {
            keys: keys(),
            help: help::new(),
            input_style: Style::new().foreground_color(Color::parse("#FF75B7")),
            last_key: String::new(),
            quitting: false,
        }
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(w) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            // If we set a width on the help menu it can gracefully truncate
            // its view as needed.
            self.help.set_width(w.width);
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let kk = &k.0;
            if key::matches(kk, std::slice::from_ref(&self.keys.up)) {
                self.last_key = "↑".to_string();
            } else if key::matches(kk, std::slice::from_ref(&self.keys.down)) {
                self.last_key = "↓".to_string();
            } else if key::matches(kk, std::slice::from_ref(&self.keys.left)) {
                self.last_key = "←".to_string();
            } else if key::matches(kk, std::slice::from_ref(&self.keys.right)) {
                self.last_key = "→".to_string();
            } else if key::matches(kk, std::slice::from_ref(&self.keys.help)) {
                self.help.show_all = !self.help.show_all;
            } else if key::matches(kk, std::slice::from_ref(&self.keys.quit)) {
                self.quitting = true;
                return quit();
            }
        }

        None
    }

    fn view(&self) -> View {
        if self.quitting {
            return View::new("Bye!\n");
        }

        let status = if self.last_key.is_empty() {
            "Waiting for input...".to_string()
        } else {
            format!("You chose: {}", self.input_style.render(&self.last_key))
        };

        let help_view = self.help.view(&self.keys);
        let height = 8usize
            .saturating_sub(status.matches('\n').count())
            .saturating_sub(help_view.matches('\n').count());

        View::new(&format!("{}{}{}", status, "\n".repeat(height), help_view))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = if std::env::var("HELP_DEBUG").is_ok() {
        match rusty_bubbletea::log_to_file("debug.log", "help") {
            Ok(f) => Some(f),
            Err(err) => {
                eprintln!("Couldn't open a file for logging: {}", err);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let p = Program::new(Model::new_model());
    p.run()?;
    Ok(())
}
