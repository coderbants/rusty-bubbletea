//! Cleanroom Rust port of upstream Go example: `examples/window-size/main.go`
//! Upstream Target Tag / Version: `v1.3.4`

use charming_bubbletea::*;

struct WindowSizeModel {}

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
            return print_f(format_args!("{}x{}", ws.width, ws.height));
        }

        None
    }

    fn view(&self) -> String {
        "When you're done press q to quit. Press any other key to query the window-size.\n".to_string()
    }
}

fn main() {
    let p = Program::new(WindowSizeModel {});
    if let Err(err) = p.run() {
        eprintln!("Error running program: {}", err);
        std::process::exit(1);
    }
}
