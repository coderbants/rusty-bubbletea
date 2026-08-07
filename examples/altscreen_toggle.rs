use charming_bubbletea::{quit, Cmd, KeyPressMsg, Model, Msg, Program, View};

struct AltScreenModel {
    alt_screen: bool,
}

impl Model for AltScreenModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let key_str = k.0.to_string();
            if key_str == "q" || key_str == "ctrl+c" {
                return quit();
            } else if key_str == "space" {
                self.alt_screen = !self.alt_screen;
            }
        }
        None
    }

    fn view(&self) -> View {
        let mut v = View::new("Press space to toggle alt screen, q to quit.");
        v.alt_screen = self.alt_screen;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(AltScreenModel { alt_screen: false });
    program.run()?;
    Ok(())
}
