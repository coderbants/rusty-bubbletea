use charming_bubbletea::{
    print_f, quit, request_window_size, Cmd, KeyPressMsg, Model, Msg, Program, View, WindowSizeMsg,
};

#[derive(Default)]
struct ModelImpl {
    width: usize,
    height: usize,
}

impl Model for ModelImpl {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            self.height = ws.height;
            print_f(format_args!("{}x{}", self.width, self.height))
        } else if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            if k.0.to_string() == "q" || k.0.to_string() == "ctrl+c" {
                quit()
            } else {
                request_window_size()
            }
        } else {
            None
        }
    }

    fn view(&self) -> View {
        View::new("When you're done press q to quit. Press any other key to query the window-size.\n")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(ModelImpl::default());
    program.run()?;
    Ok(())
}
