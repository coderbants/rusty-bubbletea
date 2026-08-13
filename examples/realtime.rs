//! Cleanroom Rust port of upstream Go example: `examples/realtime/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple example that shows how to send activity to Bubble Tea in real-time
//! through a channel.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rusty_bubbles::spinner;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{batch, quit, Cmd, KeyPressMsg, Msg, Program, View};

/// A message used to indicate that activity has occurred. In the real world
/// (for example, chat) this would contain actual data.
#[derive(Debug)]
struct ResponseMsg;

/// A tiny pseudo-random number generator used to pick the interval between
/// activity events, mirroring the upstream `math/rand` usage.
static RNG_STATE: AtomicU64 = AtomicU64::new(0x9E3779B97F4A7C15);

fn random_interval() -> Duration {
    let mut x = RNG_STATE.fetch_add(0x9E3779B97F4A7C15, Ordering::Relaxed);
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    let n = x.wrapping_mul(0x2545F4914F6CDD1D);
    // A random interval between 100 and 1000 milliseconds.
    Duration::from_millis(100 + (n >> 32) % 900)
}

/// Simulate a process that sends events at an irregular interval in real time.
/// In this case, we'll send events on the channel at a random interval between
/// 100 to 1000 milliseconds. As a command, Bubble Tea will run this
/// asynchronously.
fn listen_for_activity(sub: mpsc::Sender<()>) -> Cmd {
    Some(Box::new(move || loop {
        thread::sleep(random_interval());
        let _ = sub.send(());
    }))
}

/// A command that waits for the activity on a channel.
fn wait_for_activity(sub: Arc<Mutex<mpsc::Receiver<()>>>) -> Cmd {
    Some(Box::new(move || {
        let _ = sub.lock().unwrap().recv();
        Some(Box::new(ResponseMsg))
    }))
}

struct Model {
    sub_tx: mpsc::Sender<()>, // where we'll send activity notifications
    sub_rx: Arc<Mutex<mpsc::Receiver<()>>>, // where we'll receive activity notifications
    responses: usize,         // how many responses we've received
    spinner: spinner::Model,
    quitting: bool,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        batch(vec![
            // Start the spinner.
            {
                let tm = self.spinner.tick_msg();
                Some(Box::new(move || Some(Box::new(tm))))
            },
            listen_for_activity(self.sub_tx.clone()), // generate activity
            wait_for_activity(self.sub_rx.clone()),   // wait for activity
        ])
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().downcast_ref::<KeyPressMsg>().is_some() {
            self.quitting = true;
            return quit();
        }

        if msg.as_any().is::<ResponseMsg>() {
            self.responses += 1; // record external activity
            return wait_for_activity(self.sub_rx.clone()); // wait for next event
        }

        if msg.as_any().downcast_ref::<spinner::TickMsg>().is_some() {
            return self.spinner.update(msg);
        }

        None
    }

    fn view(&self) -> View {
        let mut s = format!(
            "\n {} Events received: {}\n\n Press any key to exit\n",
            self.spinner.view(),
            self.responses
        );
        if self.quitting {
            s += "\n";
        }
        View::new(&s)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<()>();
    let rx = Arc::new(Mutex::new(rx));

    let m = Model {
        sub_tx: tx,
        sub_rx: rx,
        responses: 0,
        spinner: spinner::new(vec![]),
        quitting: false,
    };

    let p = Program::new(m);
    p.run()?;
    Ok(())
}
