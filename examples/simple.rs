use charming_bubbletea::{quit, Cmd, KeyPressMsg, Model, Msg, Program, View};

struct SimpleModel;

impl Model for SimpleModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            if k.0.to_string() == "q" || k.0.to_string() == "ctrl+c" {
                return quit();
            }
        }
        None
    }

    fn view(&self) -> View {
        View::new("Hello from Bubble Tea v2.0.8 in Rust! Press q to quit.\n")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(SimpleModel);
    program.run()?;
    Ok(())
}
