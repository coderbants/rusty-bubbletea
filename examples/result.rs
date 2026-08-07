use charming_bubbletea::{quit, Cmd, KeyPressMsg, Model, Msg, Program, View};

struct ResultModel {
    choice: String,
}

impl Model for ResultModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let key_str = k.0.to_string();
            if key_str == "q" || key_str == "ctrl+c" {
                return quit();
            } else {
                self.choice = key_str;
            }
        }
        None
    }

    fn view(&self) -> View {
        View::new(&format!("Selected choice: {}\nPress q to quit.", self.choice))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(ResultModel {
        choice: "None".to_string(),
    });
    let final_model = program.run()?;
    println!("Final selection was: {}", final_model.choice);
    Ok(())
}
