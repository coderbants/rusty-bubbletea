//! Cleanroom Rust port of upstream Go example: `examples/send-msg/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple example that shows how to send messages to a Bubble Tea program
//! from outside the program using `Program::send`.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use rusty_bubbles::spinner;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::{new_style, Color, Style};

/// The number of most recent results to keep around.
const NUM_LAST_RESULTS: usize = 5;

/// Styles used by the example, mirroring the upstream global styles.
fn spinner_style() -> Style {
    new_style().foreground_color(Color::parse("63"))
}

fn help_style() -> Style {
    new_style()
        .foreground_color(Color::parse("241"))
        .margin(&[1, 0])
}

fn app_style() -> Style {
    new_style().margin(&[1, 2, 0, 2])
}

/// A message containing the result of "eating" some food, i.e. the pause
/// duration and the food that was eaten.
#[derive(Debug, Clone)]
struct ResultMsg {
    duration: Duration,
    food: String,
}

impl ResultMsg {
    /// Renders the result: a dotted placeholder while the duration is zero,
    /// otherwise a line describing what was eaten.
    fn to_string(&self, dot: &Style, duration: &Style) -> String {
        if self.duration == Duration::ZERO {
            return dot.render(&".".repeat(30));
        }
        format!(
            "🍔 Ate {} {}",
            self.food,
            duration.render(&format_duration(self.duration))
        )
    }
}

/// A model showing a spinner while "eating" and a rolling list of results.
struct Model {
    spinner: spinner::Model,
    results: Vec<ResultMsg>,
    quitting: bool,
}

fn new_model() -> Model {
    let s = spinner::new(vec![spinner::with_style(spinner_style())]);
    Model {
        spinner: s,
        results: vec![
            ResultMsg {
                duration: Duration::ZERO,
                food: String::new(),
            };
            NUM_LAST_RESULTS
        ],
        quitting: false,
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        // Start the spinner ticking, mirroring `m.spinner.Tick`.
        let tick = self.spinner.tick_msg();
        Some(Box::new(move || Some(Box::new(tick))))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().is::<KeyPressMsg>() {
            self.quitting = true;
            return quit();
        }

        if let Some(res) = msg.as_any().downcast_ref::<ResultMsg>() {
            // Shift the results and append the new one.
            self.results.remove(0);
            self.results.push(res.clone());
            return None;
        }

        if msg.as_any().is::<spinner::TickMsg>() {
            return self.spinner.update(msg);
        }

        None
    }

    fn view(&self) -> View {
        let mut b = String::new();

        if self.quitting {
            b.push_str("That's all for today!");
        } else {
            b.push_str(&self.spinner.view());
            b.push_str(" Eating food...");
        }

        b.push_str("\n\n");

        let dot = help_style().unset_margins();
        let duration = dot.clone();
        for res in &self.results {
            b.push_str(&res.to_string(&dot, &duration));
            b.push('\n');
        }

        if !self.quitting {
            b.push_str(&help_style().render("Press any key to exit"));
        }

        if self.quitting {
            b.push('\n');
        }

        View::new(&app_style().render(&b))
    }
}

/// A tiny xorshift64* PRNG seeded from the system clock, used in place of
/// Go's `math/rand` (deviation: no external rand dependency).
struct Rng(u64);

impl Rng {
    fn seeded() -> Rng {
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Rng(seed)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

thread_local! {
    static RNG: std::cell::RefCell<Rng> = std::cell::RefCell::new(Rng::seeded());
}

/// Random pause of 100-999ms, mirroring
/// `time.Duration(rand.Int63n(899)+100) * time.Millisecond`.
fn random_pause() -> Duration {
    let ms = RNG.with(|r| r.borrow_mut().next() % 899 + 100);
    Duration::from_millis(ms)
}

/// Random food from the same list as the upstream example.
fn random_food() -> &'static str {
    const FOOD: [&str; 13] = [
        "an apple",
        "a pear",
        "a gherkin",
        "a party gherkin",
        "a kohlrabi",
        "some spaghetti",
        "tacos",
        "a currywurst",
        "some curry",
        "a sandwich",
        "some peanut butter",
        "some cashews",
        "some ramen",
    ];
    let i = RNG.with(|r| r.borrow_mut().next() % FOOD.len() as u64) as usize;
    FOOD[i]
}

/// Formats a duration like Go's `Duration.String()` (e.g. "500ms", "1.5s",
/// "1.234s").
fn format_duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs >= 1.0 {
        let s = format!("{:.3}", secs);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        format!("{}s", s)
    } else {
        format!("{}ms", d.as_millis())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(new_model());
    let shared: Arc<Mutex<Option<Program<Model>>>> = Arc::new(Mutex::new(Some(p)));

    // Simulate activity: send a ResultMsg from outside the program at
    // random intervals, mirroring the upstream goroutine. Deviation: the
    // upstream `p.Send` blocks until the program is ready to receive
    // messages, so no message is lost; here messages sent before `run()`
    // starts are dropped instead (the first send is at least 100ms after
    // the program thread starts, so in practice none are lost).
    let activity = shared.clone();
    thread::spawn(move || loop {
        let pause = random_pause();
        thread::sleep(pause);
        let msg = ResultMsg {
            food: random_food().to_string(),
            duration: pause,
        };
        if let Some(pg) = activity.lock().unwrap().as_ref() {
            pg.send(Box::new(msg));
        }
    });

    // Run the program on a dedicated thread so the activity thread can keep
    // borrowing the program handle to inject messages.
    let runner = shared.clone();
    let handle = thread::spawn(move || {
        if let Some(pg) = runner.lock().unwrap().take() {
            let _ = pg.run();
        }
    });
    let _ = handle.join();

    Ok(())
}
