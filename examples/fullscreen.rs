use charming_bubbletea::{quit, Cmd, KeyPressMsg, Model, Msg, Program, View};

struct FullscreenModel;

impl Model for FullscreenModel {
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
        let mut v = View::new("Fullscreen app in Bubble Tea v2.0.8! Press q to exit.");
        v.alt_screen = true;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(FullscreenModel);
    program.run()?;
    Ok(())
}
