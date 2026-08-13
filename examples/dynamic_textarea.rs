//! Cleanroom Rust port of upstream Go example: `examples/dynamic-textarea/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A textarea that dynamically grows and shrinks its height to fit the
//! content, clamped between a minimum and maximum height.

use rusty_bubbles::textarea;
use rusty_bubbletea::cursor::Cursor;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{batch, quit, BackgroundColorMsg, Cmd, KeyPressMsg, Msg, Program, View};

struct Model {
    textarea: textarea::Model,
}

impl Model {
    fn initial_model() -> Self {
        let mut ti = textarea::new();
        ti.placeholder = "Schnrr...".to_string();
        ti.show_line_numbers = true;
        ti.dynamic_height = true;
        ti.min_height = 3;
        ti.max_height = 15;
        ti.max_content_height = 20;
        ti.set_width(60);
        ti.set_virtual_cursor(false);
        ti.focus();

        Model { textarea: ti }
    }

    fn status_view(&self) -> String {
        format!(
            "\nHeight: {} · Lines: {} · Cursor: ({}, {}) · Scroll: {:.0}%",
            self.textarea.height(),
            self.textarea.line_count(),
            self.textarea.line(),
            self.textarea.column(),
            self.textarea.scroll_percent() * 100.0,
        )
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        batch(vec![
            Some(Box::new(|| Some(textarea::blink()))),
            rusty_bubbletea::color::request_background_color(),
        ])
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        let mut cmds: Vec<Cmd> = Vec::new();

        if let Some(bg) = msg.as_any().downcast_ref::<BackgroundColorMsg>() {
            self.textarea
                .set_styles(textarea::default_styles(bg.is_dark()));
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            if k.0.to_string().as_str() == "ctrl+c" {
                return quit();
            }
        }

        cmds.push(self.textarea.update(msg));
        batch(cmds)
    }

    fn view(&self) -> View {
        const GAP: usize = 1;

        let mut c: Option<Cursor> = None;
        if !self.textarea.virtual_cursor() {
            if let Some(mut cur) = self.textarea.cursor() {
                cur.position.y += GAP;
                c = Some(cur);
            }
        }

        let mut f = "\n".repeat(GAP);
        f += &format!(
            "{}\n{}\n{}",
            self.textarea.view(),
            self.status_view(),
            "\n(ctrl+c to quit)",
        );

        let mut v = View::new(&f);
        v.cursor = c;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::initial_model());
    p.run()?;
    Ok(())
}
