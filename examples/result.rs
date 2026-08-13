//! Cleanroom Rust port of upstream Go example: `examples/result/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple choice menu: navigate with up/down (or k/j), press enter to
//! select, and the selection is printed after the program exits.

use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Model, Msg, Program, View};

const CHOICES: [&str; 3] = ["Taro", "Coffee", "Lychee"];

struct ResultModel {
    cursor: usize,
    choice: String,
}

impl Model for ResultModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "q" | "esc" => return quit(),
                "enter" => {
                    // Send the choice on the channel and exit.
                    self.choice = CHOICES[self.cursor].to_string();
                    return quit();
                }
                "down" | "j" => {
                    self.cursor += 1;
                    if self.cursor >= CHOICES.len() {
                        self.cursor = 0;
                    }
                }
                "up" | "k" => {
                    self.cursor = self.cursor.saturating_sub(1);
                    if self.cursor >= CHOICES.len() {
                        self.cursor = CHOICES.len() - 1;
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> View {
        let mut s = String::from("What kind of Bubble Tea would you like to order?\n\n");
        for (i, choice) in CHOICES.iter().enumerate() {
            if self.cursor == i {
                s.push_str(&format!("(•) {choice}\n"));
            } else {
                s.push_str(&format!("( ) {choice}\n"));
            }
        }
        s.push_str("\n(press q to quit)\n");
        View::new(&s)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(ResultModel {
        cursor: 0,
        choice: String::new(),
    });
    let m = program.run()?;
    // Assert the final model and print the choice.
    if !m.choice.is_empty() {
        println!("\n---\nYou chose {}!", m.choice);
    }
    Ok(())
}
