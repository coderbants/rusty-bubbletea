//! Cleanroom Rust port of upstream Go example: `examples/window-size/main.go`
//! Upstream Target Tag / Version: `v1.3.4`

use charming_bubbletea::*;

struct WindowSizeModel {
    size_info: String,
}

impl Model for WindowSizeModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(key) = msg.as_ref().as_any().downcast_ref::<KeyMsg>() {
            match key.to_string_rep().as_str() {
                "ctrl+c" | "q" | "esc" => return quit(),
                _ => return window_size(),
            }
        }

        if let Some(ws) = msg.as_ref().as_any().downcast_ref::<WindowSizeMsg>() {
            self.size_info = format!("{}x{}", ws.width, ws.height);
        }

        None
    }

    fn view(&self) -> String {
        format!(
            "When you're done press q to quit. Window size: {}\n",
            self.size_info
        )
    }
}

fn main() {
    let p = Program::new(WindowSizeModel {
        size_info: "Unknown".to_string(),
    });
    if let Err(err) = p.run() {
        eprintln!("Error running program: {}", err);
        std::process::exit(1);
    }
}
