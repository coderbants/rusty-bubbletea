use charming_bubbletea::{exec_process, quit, Cmd, KeyPressMsg, Model, Msg, Program, View};

struct ExecModel;

impl Model for ExecModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let key_str = k.0.to_string();
            if key_str == "q" || key_str == "ctrl+c" {
                return quit();
            } else if key_str == "e" {
                return exec_process("vim", &["file.txt"]);
            }
        }
        None
    }

    fn view(&self) -> View {
        View::new("Press e to spawn editor (vim), q to quit.")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(ExecModel);
    program.run()?;
    Ok(())
}
