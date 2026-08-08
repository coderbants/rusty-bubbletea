use charming_bubbletea::{quit, tick, Cmd, KeyPressMsg, Model, Msg, Program, View};
use std::time::{Duration, SystemTime};

struct DebounceModel {
    tag: u32,
}

#[derive(Debug, Clone)]
struct TagMsg(#[allow(dead_code)] u32);

impl Model for DebounceModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            if k.0.to_string() == "q" || k.0.to_string() == "ctrl+c" {
                return quit();
            } else {
                self.tag += 1;
                let current_tag = self.tag;
                return tick(Duration::from_millis(500), move |_ts: SystemTime| {
                    Some(Box::new(TagMsg(current_tag)))
                });
            }
        }
        None
    }

    fn view(&self) -> View {
        View::new("Debounce example. Press keys quickly; only settles after 500ms inactivity. Press q to quit.")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(DebounceModel { tag: 0 });
    program.run()?;
    Ok(())
}
