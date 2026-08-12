//! Cleanroom Rust port of upstream Go example: `examples/split-editors/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A program that demonstrates multiple textareas split side by side, with
//! support for adding, removing and cycling between editors.

use charming_bubbles::help;
use charming_bubbles::key::{self, Binding};
use charming_bubbles::textarea;
use charming_bubbletea::cursor::Cursor;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::screen::WindowSizeMsg;
use charming_bubbletea::{batch, quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::border;
use charming_lipgloss::join::join_horizontal;
use charming_lipgloss::{Color, Style, TOP};

const INITIAL_INPUTS: usize = 2;
const MAX_INPUTS: usize = 6;
const MIN_INPUTS: usize = 1;
const HELP_HEIGHT: usize = 5;

fn cursor_color() -> Color {
    Color::parse("212")
}

fn cursor_line_style() -> Style {
    Style::new()
        .background_color(Color::parse("57"))
        .foreground_color(Color::parse("230"))
}

fn placeholder_style() -> Style {
    Style::new().foreground_color(Color::parse("238"))
}

fn focused_placeholder_style() -> Style {
    Style::new().foreground_color(Color::parse("99"))
}

fn focused_border_style() -> Style {
    Style::new()
        .border(border::rounded_border(), &[true, true, true, true])
        .border_foreground(&["238"])
}

fn blurred_border_style() -> Style {
    Style::new().border(border::hidden_border(), &[true, true, true, true])
}

fn end_of_buffer_style() -> Style {
    Style::new().foreground_color(Color::parse("235"))
}

fn new_textarea() -> textarea::Model {
    let mut t = textarea::new();
    t.prompt = String::new();
    t.placeholder = "Type something".to_string();
    t.show_line_numbers = true;
    t.set_virtual_cursor(true);

    let mut s = t.styles().clone();
    s.cursor.color = cursor_color();
    s.focused.placeholder = focused_placeholder_style();
    s.blurred.placeholder = placeholder_style();
    s.focused.cursor_line = cursor_line_style();
    s.focused.cursor_line_number = cursor_line_style();
    s.focused.base = focused_border_style();
    s.blurred.base = blurred_border_style();
    s.focused.end_of_buffer = end_of_buffer_style();
    s.blurred.end_of_buffer = end_of_buffer_style();
    t.set_styles(s);

    t.key_map.delete_word_backward.set_enabled(false);
    t.key_map.line_next = key::new_binding(vec![key::with_keys(&["down"])]);
    t.key_map.line_previous = key::new_binding(vec![key::with_keys(&["up"])]);
    t.blur();
    t
}

struct Keymap {
    next: Binding,
    prev: Binding,
    add: Binding,
    remove: Binding,
    quit: Binding,
}

struct Model {
    width: usize,
    height: usize,
    keymap: Keymap,
    help: help::Model,
    inputs: Vec<textarea::Model>,
    focus: usize,
}

impl Model {
    fn new_model() -> Self {
        let mut m = Model {
            width: 0,
            height: 0,
            keymap: Keymap {
                next: key::new_binding(vec![
                    key::with_keys(&["tab"]),
                    key::with_help("tab", "next"),
                ]),
                prev: key::new_binding(vec![
                    key::with_keys(&["shift+tab"]),
                    key::with_help("shift+tab", "prev"),
                ]),
                add: key::new_binding(vec![
                    key::with_keys(&["ctrl+n"]),
                    key::with_help("ctrl+n", "add an editor"),
                ]),
                remove: key::new_binding(vec![
                    key::with_keys(&["ctrl+w"]),
                    key::with_help("ctrl+w", "remove an editor"),
                ]),
                quit: key::new_binding(vec![
                    key::with_keys(&["esc", "ctrl+c"]),
                    key::with_help("esc", "quit"),
                ]),
            },
            help: help::new(),
            inputs: vec![],
            focus: 0,
        };
        for _ in 0..INITIAL_INPUTS {
            m.inputs.push(new_textarea());
        }
        m.inputs[m.focus].focus();
        m.update_keybindings();
        m
    }

    fn size_inputs(&mut self) {
        let n = self.inputs.len();
        let w = self.width / n;
        let h = self.height.saturating_sub(HELP_HEIGHT);
        for i in 0..n {
            self.inputs[i].set_width(w);
            self.inputs[i].set_height(h);
        }
    }

    fn update_keybindings(&mut self) {
        self.keymap.add.set_enabled(self.inputs.len() < MAX_INPUTS);
        self.keymap
            .remove
            .set_enabled(self.inputs.len() > MIN_INPUTS);
    }

    fn input_views(&self) -> Vec<String> {
        let mut views = Vec::new();
        for i in 0..self.inputs.len() {
            views.push(self.inputs[i].view());
        }
        views
    }

    /// Cursor returns the real cursor position for the focused editor,
    /// accounting for the width of the editors to its left.
    fn cursor(&self) -> Option<Cursor> {
        let focused_input = &self.inputs[self.focus];
        if focused_input.virtual_cursor() {
            return None;
        }

        let views = self.input_views();
        let mut c = focused_input.cursor()?;

        // Find textarea offset to position the real cursor: calculate the
        // width of all textareas to the left of the focused one.
        for v in views.iter().take(self.focus) {
            c.position.x += charming_lipgloss::size::width(v);
        }

        Some(c)
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        Some(Box::new(|| Some(textarea::blink())))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        let mut cmds: Vec<Cmd> = Vec::new();

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let s = k.0.to_string();
            if key::matches(&s, std::slice::from_ref(&self.keymap.quit)) {
                for i in 0..self.inputs.len() {
                    self.inputs[i].blur();
                }
                return quit();
            } else if key::matches(&s, std::slice::from_ref(&self.keymap.next)) {
                self.inputs[self.focus].blur();
                self.focus += 1;
                if self.focus > self.inputs.len() - 1 {
                    self.focus = 0;
                }
                cmds.push(self.inputs[self.focus].focus());
            } else if key::matches(&s, std::slice::from_ref(&self.keymap.prev)) {
                self.inputs[self.focus].blur();
                if self.focus == 0 {
                    self.focus = self.inputs.len() - 1;
                } else {
                    self.focus -= 1;
                }
                cmds.push(self.inputs[self.focus].focus());
            } else if key::matches(&s, std::slice::from_ref(&self.keymap.add)) {
                self.inputs.push(new_textarea());
            } else if key::matches(&s, std::slice::from_ref(&self.keymap.remove)) {
                self.inputs.pop();
                if self.focus > self.inputs.len() - 1 {
                    self.focus = self.inputs.len() - 1;
                }
            }
        }

        if let Some(w) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.height = w.height;
            self.width = w.width;
        }

        self.update_keybindings();
        self.size_inputs();

        // Update all textareas.
        for i in 0..self.inputs.len() {
            cmds.push(self.inputs[i].update(msg));
        }

        batch(cmds)
    }

    fn view(&self) -> View {
        let help_view = self.help.short_help_view(&[
            self.keymap.next.clone(),
            self.keymap.prev.clone(),
            self.keymap.add.clone(),
            self.keymap.remove.clone(),
            self.keymap.quit.clone(),
        ]);

        let views = self.input_views();
        let joined = join_horizontal(
            TOP,
            &views.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        );

        let mut v = View::new(&(joined + "\n\n" + &help_view));
        v.alt_screen = true;
        // The editors use the virtual cursor, so this is None in practice;
        // mirror the upstream helper regardless.
        v.cursor = self.cursor();
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::new_model());
    p.run()?;
    Ok(())
}
