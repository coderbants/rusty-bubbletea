use charming_bubbletea::{quit, sequence, Cmd, KeyPressMsg, Model, Msg, Program, View};

struct SequenceModel;

impl Model for SequenceModel {
    fn init(&self) -> Cmd {
        sequence(vec![None, None])
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
        View::new("Sequence example running in Bubble Tea v2.0.8! Press q to quit.")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(SequenceModel);
    program.run()?;
    Ok(())
}
