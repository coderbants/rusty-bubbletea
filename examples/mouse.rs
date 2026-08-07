//! Cleanroom Rust port of upstream Go example: `examples/mouse/main.go`
//! Upstream Target Tag / Version: `v1.3.4`

use charming_bubbletea::*;

struct MouseModel {
    last_event: String,
}

impl Model for MouseModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(key) = msg.as_ref().as_any().downcast_ref::<KeyMsg>() {
            match key.to_string_rep().as_str() {
                "ctrl+c" | "q" | "esc" => return quit(),
                _ => {}
            }
        }

        if let Some(mouse) = msg.as_ref().as_any().downcast_ref::<MouseMsg>() {
            self.last_event = format!("(X: {}, Y: {}) {}", mouse.x, mouse.y, mouse);
        }

        None
    }

    fn view(&self) -> String {
        format!(
            "Do mouse stuff. When you're done press q to quit.\nLast event: {}\n",
            self.last_event
        )
    }
}

fn main() {
    let p = Program::new(MouseModel {
        last_event: String::new(),
    });
    if let Err(err) = p.run() {
        eprintln!("Error running program: {}", err);
        std::process::exit(1);
    }
}
