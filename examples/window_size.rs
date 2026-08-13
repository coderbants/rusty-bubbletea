use rusty_bubbletea::{
    print_f, quit, request_window_size, Cmd, KeyPressMsg, Model, Msg, Program, View, WindowSizeMsg,
};

// A simple program that queries and displays the window-size.

#[derive(Default)]
struct ModelImpl {}

impl Model for ModelImpl {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let s = k.0.to_string();
            if s == "ctrl+c" || s == "q" || s == "esc" {
                return quit();
            }
            return request_window_size();
        }

        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            return print_f(format_args!(
                "The window size is: {}x{}",
                ws.width, ws.height
            ));
        }

        None
    }

    fn view(&self) -> View {
        View::new(
            "\nWhen you're done press q to quit.\nPress any other key to query the window-size.\n",
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(ModelImpl::default());
    program.run()?;
    Ok(())
}
