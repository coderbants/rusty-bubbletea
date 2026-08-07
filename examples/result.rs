//! Cleanroom Rust port of upstream Go example: `examples/result/main.go`
//! Upstream Target Tag / Version: `v1.3.4`

use charming_bubbletea::*;

const CHOICES: &[&str] = &["Taro", "Coffee", "Lychee"];

struct ResultModel {
    cursor: usize,
    choice: String,
}

impl Model for ResultModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(key) = msg.as_ref().as_any().downcast_ref::<KeyMsg>() {
            match key.to_string_rep().as_str() {
                "ctrl+c" | "q" | "esc" => return quit(),
                "enter" => {
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
                    if self.cursor == 0 {
                        self.cursor = CHOICES.len() - 1;
                    } else {
                        self.cursor -= 1;
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> String {
        let mut s = String::from("What kind of Bubble Tea would you like to order?\n\n");
        for (i, choice) in CHOICES.iter().enumerate() {
            if self.cursor == i {
                s.push_str("(•) ");
            } else {
                s.push_str("( ) ");
            }
            s.push_str(choice);
            s.push('\n');
        }
        s.push_str("\n(press q to quit)\n");
        s
    }
}

fn main() {
    let model = ResultModel {
        cursor: 0,
        choice: String::new(),
    };
    let p = Program::new(model);
    match p.run() {
        Ok(final_model) => {
            if !final_model.choice.is_empty() {
                println!("\n---\nYou chose {}!\n", final_model.choice);
            }
        }
        Err(err) => {
            eprintln!("Oh no: {}", err);
            std::process::exit(1);
        }
    }
}
