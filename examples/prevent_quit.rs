//! Cleanroom Rust port of upstream Go example: `examples/prevent-quit/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A program demonstrating how to use the WithFilter option to intercept
//! events (specifically, preventing accidental quits when there are unsaved
//! changes).

use charming_bubbles::help;
use charming_bubbles::key::{self, Binding};
use charming_bubbles::textarea;
use charming_bubbletea::commands::QuitMsg;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::options::ProgramOptions;
use charming_bubbletea::{batch, quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::border;
use charming_lipgloss::join::join_horizontal;
use charming_lipgloss::{new_style, Color, Style, TOP};

/// Styles used by the example, mirroring the upstream global styles.
fn choice_style() -> Style {
    new_style()
        .padding_left(1)
        .foreground_color(Color::parse("241"))
}

fn save_text_style() -> Style {
    new_style().foreground_color(Color::parse("170"))
}

fn quit_view_style() -> Style {
    new_style()
        .padding(&[1, 3])
        .border(border::rounded_border(), &[true, true, true, true])
        .border_foreground(&["170"])
}

/// Key bindings for the example.
struct Keymap {
    save: Binding,
    quit: Binding,
}

/// The main model, wrapping a textarea plus help and keymap state.
struct Model {
    textarea: textarea::Model,
    help: help::Model,
    keymap: Keymap,
    save_text: String,
    has_changes: bool,
    quitting: bool,
}

/// The initial model: a focused textarea with a placeholder.
fn initial_model() -> Model {
    let mut ti = textarea::new();
    ti.placeholder = "Only the best words".to_string();
    ti.focus();

    Model {
        textarea: ti,
        help: help::new(),
        keymap: Keymap {
            save: key::new_binding(vec![
                key::with_keys(&["ctrl+s"]),
                key::with_help("ctrl+s", "save"),
            ]),
            quit: key::new_binding(vec![
                key::with_keys(&["esc", "ctrl+c"]),
                key::with_help("esc", "quit"),
            ]),
        },
        save_text: String::new(),
        has_changes: false,
        quitting: false,
    }
}

/// The event filter: swallow `QuitMsg`s while there are unsaved changes,
/// mirroring the upstream `filter` function.
fn filter(m: &Model, msg: Box<dyn Msg>) -> Option<Box<dyn Msg>> {
    if !msg.as_ref().as_any().is::<QuitMsg>() {
        return Some(msg);
    }

    if m.has_changes {
        return None;
    }

    Some(msg)
}

impl Model {
    /// Update loop for the textarea view.
    fn update_text_view(&mut self, msg: &dyn Msg) -> Cmd {
        let mut cmds: Vec<Cmd> = Vec::new();

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            self.save_text = String::new();
            let s = k.0.to_string();
            if key::matches(&s, std::slice::from_ref(&self.keymap.save)) {
                self.save_text = "Changes saved!".to_string();
                self.has_changes = false;
            } else if key::matches(&s, std::slice::from_ref(&self.keymap.quit)) {
                self.quitting = true;
                return quit();
            } else {
                if !k.0.text.is_empty() {
                    self.save_text = String::new();
                    self.has_changes = true;
                }
                if !self.textarea.focused() {
                    cmds.push(self.textarea.focus());
                }
            }
        }

        cmds.push(self.textarea.update(msg));
        batch(cmds)
    }

    /// Update loop for the "are you sure?" prompt view.
    fn update_prompt_view(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            // For simplicity's sake, we'll treat any key besides "y" as "no".
            let s = k.0.to_string();
            if key::matches(&s, std::slice::from_ref(&self.keymap.quit)) || s == "y" {
                self.has_changes = false;
                return quit();
            }
            self.quitting = false;
        }

        None
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        // Start the cursor blinking, mirroring `textarea.Blink`.
        Some(Box::new(|| Some(textarea::blink())))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if self.quitting {
            return self.update_prompt_view(msg);
        }

        self.update_text_view(msg)
    }

    fn view(&self) -> View {
        if self.quitting {
            if self.has_changes {
                let text = join_horizontal(
                    TOP,
                    &[
                        "You have unsaved changes. Quit without saving?",
                        &choice_style().render("[yN]"),
                    ],
                );
                return View::new(&quit_view_style().render(&text));
            }
            return View::new("Very important. Thank you.\n");
        }

        let help_view = self
            .help
            .short_help_view(&[self.keymap.save.clone(), self.keymap.quit.clone()]);

        View::new(
            &(format!(
                "Type some important things.\n{}\n {}\n {}",
                self.textarea.view(),
                save_text_style().render(&self.save_text),
                help_view,
            ) + "\n\n"),
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(initial_model())
        .with_options(ProgramOptions::default().with_filter(Box::new(filter)));
    p.run()?;
    Ok(())
}
