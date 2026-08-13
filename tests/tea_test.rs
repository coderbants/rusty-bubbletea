use rusty_bubbletea::{quit, Cmd, Model, Msg, Program, View};

struct TestModel {
    counter: usize,
}

impl Model for TestModel {
    fn init(&self) -> Cmd {
        // Immediately send a quit so the program exits without needing a real TTY.
        quit()
    }

    fn update(&mut self, _msg: &dyn Msg) -> Cmd {
        self.counter += 1;
        quit()
    }

    fn view(&self) -> View {
        View::new(&format!("Counter: {}", self.counter))
    }
}

/// Full interactive program run — requires a real TTY, skip in CI.
#[test]
#[ignore]
fn test_v2_program_run() {
    let model = TestModel { counter: 0 };
    let prog = Program::new(model);
    assert_eq!(prog.run().unwrap().counter, 0);
}
