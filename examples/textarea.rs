//! Cleanroom Rust port of upstream Go example: `examples/textarea/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple program demonstrating the textarea component from the Bubbles
//! component library.

use rusty_bubbles::textarea;
use rusty_bubbletea::cursor::Cursor;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{batch, quit, BackgroundColorMsg, Cmd, KeyPressMsg, Msg, Program, View};

/// An error message, mirroring the upstream `errMsg` type.
#[derive(Debug)]
struct ErrMsg;

struct Model {
    textarea: textarea::Model,
    err: Option<String>,
}

impl Model {
    fn initial_model() -> Self {
        let mut ti = textarea::new();
        ti.placeholder = "Once upon a time...".to_string();
        ti.set_virtual_cursor(false);
        ti.set_styles(textarea::default_styles(true)); // default to dark styles.
        ti.focus();

        Model {
            textarea: ti,
            err: None,
        }
    }

    fn header_view(&self) -> String {
        "Tell me a story.\n".to_string()
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
            // Update styling now that we know the background color.
            self.textarea
                .set_styles(textarea::default_styles(bg.is_dark()));
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "esc" => {
                    if self.textarea.focused() {
                        self.textarea.blur();
                    }
                }
                "ctrl+c" => return quit(),
                _ => {
                    if !self.textarea.focused() {
                        cmds.push(self.textarea.focus());
                    }
                }
            }
        }

        // We handle errors just like any other message.
        if msg.as_any().is::<ErrMsg>() {
            self.err = Some("(error)".to_string());
            return None;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            use std::io::Write as _;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/ta.log")
                .unwrap();
            let _ = writeln!(
                f,
                "key={:?} focused={} val={:?}",
                k.0.to_string(),
                self.textarea.focused(),
                self.textarea.value()
            );
        }
        cmds.push(self.textarea.update(msg));
        batch(cmds)
    }

    fn view(&self) -> View {
        const FOOTER: &str = "\n(ctrl+c to quit)\n";

        let mut c: Option<Cursor> = None;
        if !self.textarea.virtual_cursor() {
            if let Some(mut cur) = self.textarea.cursor() {
                // Set the y offset of the cursor based on the position of the
                // textarea in the application.
                let offset = rusty_lipgloss::size::height(&self.header_view());
                cur.position.y += offset;
                c = Some(cur);
            }
        }

        let f = format!(
            "{}\n{}\n{}",
            self.header_view(),
            self.textarea.view(),
            FOOTER
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
