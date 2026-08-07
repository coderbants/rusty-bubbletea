//! Cleanroom Rust port of upstream Go example: `examples/simple/main.go`
//! Upstream Target Tag / Version: `v1.3.4`

use charming_bubbletea::*;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct TickMsg;

struct SimpleModel {
    counter: i32,
}

impl Model for SimpleModel {
    fn init(&self) -> Cmd {
        Some(Box::new(|| {
            thread::sleep(Duration::from_secs(1));
            Some(Box::new(TickMsg))
        }))
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(key) = msg.as_ref().as_any().downcast_ref::<KeyMsg>() {
            match key.to_string_rep().as_str() {
                "ctrl+c" | "q" => return quit(),
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
            "Hi. This program will exit in {} seconds.\n\nTo quit sooner press q or ctrl+c...\n",
            self.counter
        )
    }
}

fn main() {
    let model = SimpleModel { counter: 5 };
    let p = Program::new(model);
    if let Err(err) = p.run() {
        eprintln!("Error running program: {}", err);
        std::process::exit(1);
    }
}
