//! Cleanroom Rust port of upstream Go example: `examples/debounce/main.go`
//! Upstream Target Tag / Version: `v1.3.4`

use charming_bubbletea::*;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExitMsg(usize);

struct DebounceModel {
    tag: usize,
}

impl Model for DebounceModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if msg.as_ref().as_any().is::<KeyMsg>() {
            self.tag += 1;
            let current_tag = self.tag;
            return Some(Box::new(move || {
                thread::sleep(Duration::from_secs(1));
                Some(Box::new(ExitMsg(current_tag)))
            }));
        }

        if let Some(exit_msg) = msg.as_ref().as_any().downcast_ref::<ExitMsg>() {
            if exit_msg.0 == self.tag {
                return quit();
            }
        }

        None
    }

    fn view(&self) -> String {
        format!(
            "Key presses: {}\nTo exit press any key, then wait for one second without pressing anything.\n",
            self.tag
        )
    }
}

fn main() {
    let p = Program::new(DebounceModel { tag: 0 });
    if let Err(err) = p.run() {
        eprintln!("Error running program: {}", err);
        std::process::exit(1);
    }
}
