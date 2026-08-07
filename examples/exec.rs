//! Cleanroom Rust port of upstream Go example: `examples/exec/main.go`
//! Upstream Target Tag / Version: `v1.3.4`

use charming_bubbletea::*;
use std::process::Command;

#[derive(Debug)]
struct EditorFinishedMsg {
    err_msg: Option<String>,
}

struct ExecModel {
    altscreen_active: bool,
    err_msg: Option<String>,
}

impl Model for ExecModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(key) = msg.as_ref().as_any().downcast_ref::<KeyMsg>() {
            match key.to_string_rep().as_str() {
                "a" => {
                    self.altscreen_active = !self.altscreen_active;
                    return if self.altscreen_active {
                        enter_alt_screen()
                    } else {
                        exit_alt_screen()
                    };
                }
                "e" => {
                    return Some(Box::new(|| {
                        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
                        let status = Command::new(editor).status();
                        let err_msg = status.err().map(|e| e.to_string());
                        Some(Box::new(EditorFinishedMsg { err_msg }))
                    }));
                }
                "ctrl+c" | "q" => return quit(),
                _ => {}
            }
        }

        if let Some(finished) = msg.as_ref().as_any().downcast_ref::<EditorFinishedMsg>() {
            if finished.err_msg.is_some() {
                self.err_msg = finished.err_msg.clone();
                return quit();
            }
        }

        None
    }

    fn view(&self) -> String {
        if let Some(err) = &self.err_msg {
            return format!("Error: {}\n", err);
        }
        "Press 'e' to open your EDITOR.\nPress 'a' to toggle the altscreen\nPress 'q' to quit.\n"
            .to_string()
    }
}

fn main() {
    let p = Program::new(ExecModel {
        altscreen_active: false,
        err_msg: None,
    });
    if let Err(err) = p.run() {
        eprintln!("Error running program: {}", err);
        std::process::exit(1);
    }
}
