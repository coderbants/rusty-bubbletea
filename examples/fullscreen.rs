//! Cleanroom Rust port of upstream Go example: `examples/fullscreen/main.go`
//! Upstream Target Tag / Version: `v1.3.4`

use charming_bubbletea::*;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct TickMsg;

struct FullscreenModel {
    counter: i32,
}

impl Model for FullscreenModel {
    fn init(&self) -> Cmd {
        Some(Box::new(|| {
            thread::sleep(Duration::from_secs(1));
            Some(Box::new(TickMsg))
        }))
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(key) = msg.as_ref().as_any().downcast_ref::<KeyMsg>() {
            match key.to_string_rep().as_str() {
                "q" | "esc" | "ctrl+c" => return quit(),
                _ => {}
            }
        }

        if msg.as_ref().as_any().is::<TickMsg>() {
            self.counter -= 1;
            if self.counter <= 0 {
                return quit();
            }
            return Some(Box::new(|| {
                thread::sleep(Duration::from_secs(1));
                Some(Box::new(TickMsg))
            }));
        }

        None
    }

    fn view(&self) -> String {
        format!(
            "\n\n     Hi. This program will exit in {} seconds...",
            self.counter
        )
    }
}

fn main() {
    let p = Program::new(FullscreenModel { counter: 5 });
    if let Err(err) = p.run() {
        eprintln!("Error running program: {}", err);
        std::process::exit(1);
    }
}
