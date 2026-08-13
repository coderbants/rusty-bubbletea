//! Cleanroom Rust port of upstream Go example: `examples/cursor-style/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple program demonstrating how to change the terminal cursor style
//! (block, underline, bar) with the left/right arrow keys.

use rusty_bubbletea::cursor::{Cursor, CursorShape};
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};

/// A model holding the current cursor shape and blink state.
struct Model {
    cursor: Cursor,
    blink: bool,
}

impl Model {
    fn new() -> Self {
        Model {
            cursor: Cursor::new(0, 0),
            blink: true,
        }
    }

    /// Describe the current cursor as a human-readable string, e.g.
    /// "blinking block".
    fn describe_cursor(&self) -> String {
        let adj = if self.blink { "blinking" } else { "steady" };
        let noun = match self.cursor.shape {
            CursorShape::CursorBlock => "block",
            CursorShape::CursorUnderline => "underline",
            CursorShape::CursorBar => "bar",
        };
        format!("{} {}", adj, noun)
    }
}

/// Previous shape, wrapping around, mirroring `m.cursor.Shape--` (with the
/// wrap from CursorBlock back to CursorBar).
fn previous_shape(shape: CursorShape) -> CursorShape {
    match shape {
        CursorShape::CursorBlock => CursorShape::CursorBar,
        CursorShape::CursorUnderline => CursorShape::CursorBlock,
        CursorShape::CursorBar => CursorShape::CursorUnderline,
    }
}

/// Next shape, wrapping around, mirroring `m.cursor.Shape++` (with the wrap
/// from CursorBar back to CursorBlock).
fn next_shape(shape: CursorShape) -> CursorShape {
    match shape {
        CursorShape::CursorBlock => CursorShape::CursorUnderline,
        CursorShape::CursorUnderline => CursorShape::CursorBar,
        CursorShape::CursorBar => CursorShape::CursorBlock,
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "q" => {
                    // Mirror upstream: the quit keys return early, BEFORE
                    // the blink toggle at the end of Update.
                    return quit();
                }
                "h" | "left" => self.cursor.shape = previous_shape(self.cursor.shape),
                "l" | "right" => self.cursor.shape = next_shape(self.cursor.shape),
                _ => {}
            }
        }
        self.blink = !self.blink;
        None
    }

    fn view(&self) -> View {
        let mut v = View::new(&format!(
            "Press left/right to change the cursor style, q or ctrl+c to quit.\n\n  <- This is the cursor (a {})",
            self.describe_cursor()
        ));
        let mut c = Cursor::new(0, 2);
        c.shape = self.cursor.shape;
        c.blink = self.blink;
        v.cursor = Some(c);
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::new());
    p.run()?;
    Ok(())
}
