//! Cleanroom Rust port of upstream Go example: `examples/chat/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple program demonstrating the text area component from the Bubbles
//! component library: a chat window with a message log viewport and a
//! textarea input.

use charming_bubbles::cursor::BlinkMsg;
use charming_bubbles::textarea;
use charming_bubbles::viewport;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::screen::WindowSizeMsg;
use charming_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::new_style;
use charming_lipgloss::{Color, Style};

struct Model {
    viewport: viewport::Model,
    messages: Vec<String>,
    textarea: textarea::Model,
    sender_style: Style,
}

impl Model {
    fn initial_model() -> Self {
        let mut ta = textarea::new();
        ta.placeholder = "Send a message...".to_string();
        ta.set_virtual_cursor(false);
        let _ = ta.focus();

        ta.prompt = "┃ ".to_string();
        ta.char_limit = 280;

        ta.set_width(30);
        ta.set_height(3);

        // Remove cursor line styling.
        let mut s = ta.styles().clone();
        s.focused.cursor_line = new_style();
        ta.set_styles(s);

        ta.show_line_numbers = false;

        let mut vp = viewport::new(vec![viewport::with_width(30), viewport::with_height(5)]);
        vp.set_content("Welcome to the chat room!\nType a message and press Enter to send.");
        vp.key_map.left.set_enabled(false);
        vp.key_map.right.set_enabled(false);

        ta.key_map.insert_newline.set_enabled(false);

        Model {
            textarea: ta,
            messages: vec![],
            viewport: vp,
            sender_style: new_style().foreground_color(Color::parse("5")),
        }
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        Some(Box::new(|| Some(textarea::blink())))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.viewport.set_width(ws.width);
            self.textarea.set_width(ws.width);
            self.viewport.set_height(ws.height - self.textarea.height());

            if !self.messages.is_empty() {
                // Wrap content before setting it.
                let content = new_style()
                    .width(self.viewport.width())
                    .render(&self.messages.join("\n"));
                self.viewport.set_content(&content);
            }
            self.viewport.goto_bottom();
            return None;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "esc" => {
                    println!("{}", self.textarea.value());
                    return quit();
                }
                "enter" => {
                    self.messages
                        .push(self.sender_style.render("You: ") + &self.textarea.value());
                    let content = new_style()
                        .width(self.viewport.width())
                        .render(&self.messages.join("\n"));
                    self.viewport.set_content(&content);
                    self.textarea.reset();
                    self.viewport.goto_bottom();
                    return None;
                }
                _ => {
                    // Send all other keypresses to the textarea.
                    return self.textarea.update(msg);
                }
            }
        }

        if msg.as_any().is::<BlinkMsg>() {
            // Textarea should also process cursor blinks.
            return self.textarea.update(msg);
        }

        None
    }

    fn view(&self) -> View {
        let viewport_view = self.viewport.view();
        let mut v = View::new(&(viewport_view.clone() + "\n" + &self.textarea.view()));
        if let Some(mut c) = self.textarea.cursor() {
            c.position.y += charming_lipgloss::size::height(&viewport_view);
            v.cursor = Some(c);
        }
        v.alt_screen = true;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::initial_model());
    p.run()?;
    Ok(())
}
