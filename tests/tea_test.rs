use charming_bubbletea::*;

#[derive(Default)]
struct TestModel {
    counter: i32,
}

#[derive(Debug)]
struct IncrementMsg;

impl Model for TestModel {
    fn init(&self) -> Cmd {
        Some(Box::new(|| Some(Box::new(IncrementMsg))))
    }

    fn update(&mut self, msg: Box<dyn Msg>) -> Cmd {
        if msg.as_ref().as_any().is::<IncrementMsg>() {
            self.counter += 1;
            return quit();
        }
        None
    }

    fn view(&self) -> String {
        format!("Counter: {}", self.counter)
    }
}

#[test]
fn test_tea_program_run() {
    let model = TestModel::default();
    let p = Program::new(model);
    let final_model = p.run().unwrap();
    assert_eq!(final_model.counter, 1);
}
