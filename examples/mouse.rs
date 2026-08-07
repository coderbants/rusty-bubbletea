use charming_bubbletea::{
    quit, Cmd, KeyPressMsg, Model, MouseClickMsg, MouseMode, Msg, Program, View,
};

#[derive(Default)]
struct MouseModel {
    last_event: String,
}

impl Model for MouseModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            if k.0.to_string() == "q" || k.0.to_string() == "ctrl+c" {
                return quit();
            }
        } else if let Some(m) = msg.as_any().downcast_ref::<MouseClickMsg>() {
            self.last_event = format!("Clicked at ({}, {}) button {:?}", m.0.x, m.0.y, m.0.button);
        }
        None
    }

    fn view(&self) -> View {
        let mut v = View::new(&format!(
            "Mouse tracking active! Last event: {}\nPress q to quit.",
            self.last_event
        ));
        v.mouse_mode = MouseMode::MouseModeAllMotion;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(MouseModel::default());
    program.run()?;
    Ok(())
}
