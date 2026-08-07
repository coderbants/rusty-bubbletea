//! Cleanroom Rust port of upstream Go example: `examples/sequence/main.go`
//! Upstream Target Tag / Version: `v1.3.4`

use charming_bubbletea::*;

struct SequenceModel;

impl Model for SequenceModel {
    fn init(&self) -> Cmd {
        sequence(vec![
            batch(vec![quit()]),
            quit(),
        ])
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if msg.as_ref().as_any().is::<KeyMsg>() {
            return quit();
        }
        None
    }

    fn view(&self) -> String {
        String::new()
    }
}

fn main() {
    let p = Program::new(SequenceModel);
    if let Err(err) = p.run() {
        eprintln!("Error running program: {}", err);
        std::process::exit(1);
    }
}
